/// Matches module (called `mtch` for rust reasons)
///
/// The #plan for matches and match sequencing
///
/// MATCHES
/// - Match configurations are stored in the DB
/// - Matches can have a "predecessor" from which the population is pulled from
/// - Match populations (players) are prepared/created before the match actually runs
///
/// SEEDING NEW MATCHES
/// - Over the weekend, "setup" a new match that has no predecessor and a large player count
/// - This match will then have lots of players generated for it
/// - The match will then be scheduled but not run until the Monday.
/// - Add queries and UI such that players can see the next upcoming match.
pub mod config;
pub mod presenter;
pub mod tick;

use anyhow::Context;
pub use config::*;

use rand::Rng;
use serde::Serialize;
use tokio::sync::broadcast::Sender;
use tracing::info;

use crate::{
    entity::{
        gen::generate_player, snapshot::EntityView, world::EntityWorld, Entity, EntityAttributes,
        EntityManager, EntityManagerMutation,
    },
    event::{EventStore, EventsView, GameEvent},
    has_markers,
    location::{generate_locations_for_world, Biome},
    logs::GameLog,
    mtch::presenter::{generate_collector, generate_presenter},
    Db, ServerCtx,
};

/// Id for a given match
/// (generated as a UUID but its just TEXT, can be anything...)
pub type MatchId = String;

/// An id identifying a specific tick
/// NOTE: Not scoped to a match but global for the server
/// NOTE: Tick ids are not unique and may overflow, just helps with debugging and testing
pub type TickId = usize;

/// The context that actions are resolved in
/// basically, points at stuff on the match
#[derive(Debug)]
pub struct ActionCtx<'a> {
    pub entities: &'a EntityView<'a>,
    pub events: &'a EventsView<'a>,
    pub config: &'a MatchConfig,
    pub world_state: &'a EntityWorld,

    log_tx: &'a Sender<GameLog>,
    events_buffer: &'a mut Vec<GameEvent>,
}

impl ActionCtx<'_> {
    pub fn send_log(&self, log: GameLog) {
        self.log_tx.send(log).unwrap();
    }

    pub fn add_event(&mut self, event: GameEvent) {
        self.events_buffer.push(event);
    }
}

pub struct MatchManager {
    pub config: MatchConfig,
    pub entities: EntityManager,
    pub events: EventStore,
}

impl MatchManager {
    pub async fn load_match(match_config: MatchConfig, db: &Db) -> Self {
        // Create an entity manager and load the entities for the match
        let mut match_entities = EntityManager::new(&match_config.match_id);
        match_entities.load_entities(db).await;

        Self {
            config: match_config,
            entities: match_entities,
            events: Default::default(),
        }
    }

    /// Load in a match configuration, generating any resources needed for the game
    ///
    /// This should only be done once per match, realistically - so prob do it when
    /// the config is created
    pub async fn initialise_new_match(&mut self, _db: &Db) -> anyhow::Result<()> {
        // Now we initialise it...
        info!("Initialising match {}", &self.config.match_id);

        // TODO: Add all the unescaped players from the last game
        // In practice, this just means cloning the entity into the new match
        let existing_players = 0;
        // if let Some(preceding_match_id) = &self.match_config.preceding_match_id {
        //     EntityManager::load_entities_from_match(preceding_match_id, db)
        //         .await
        //         .filter(|e| has_markers!(e, Player) && !has_markers!(e, Escaped))
        //         .for_each(|entity| {
        //             existing_players += 1;
        //             unimplemented!(); // TODO: actually copy across the entity
        //         })
        // }

        // If we dont have enough players for the match configuration,
        // then generate and add more
        let player_count_to_gen = self.config.player_count - existing_players;
        for _ in 0..player_count_to_gen {
            let mut player_entity = generate_player().context("Generating player entity")?;

            // Remove the player hex so they are effectively "banished" until we "warp them in"
            player_entity.attributes.hex = None;

            // And add them
            self.entities.upsert_entity(player_entity)?;
        }

        // Generate a location entity in each hex
        let mut rng = rand::rng();
        for entity in generate_locations_for_world(self.config.world_radius as isize, Biome::Green)
        {
            // Create the location
            self.entities.upsert_entity(entity.clone())?;

            // Generate some amount of props in each hex
            let hex = entity.attributes.hex.as_ref().unwrap();
            let location_kind = entity.attributes.location.as_ref().unwrap().location_kind;
            let prop_generators = location_kind.prop_generators();
            let max_gen = prop_generators.max_count.unwrap_or(5);
            let prop_count = rng.random_range(0..=max_gen);

            // Generate required entities for location type
            for required_generator in &prop_generators.required {
                let mut entity = required_generator.generate(&mut rng);
                // Set its location and insert it
                entity.attributes.hex = Some(*hex);
                self.entities.upsert_entity(entity)?;
            }

            // Generate a few from the optional generators
            if !prop_generators.optional.is_empty() {
                for _ in 0..prop_count {
                    let entity = prop_generators.generate_optional_at(*hex, &mut rng);
                    self.entities.upsert_entity(entity)?;
                }
            }
        }

        // Establish the current state of the world
        self.entities.upsert_entity(Entity {
            entity_id: Entity::id(),
            name: "World".into(),
            attributes: EntityAttributes {
                world: Some(EntityWorld::default()),
                ..Default::default()
            },
            ..Default::default()
        })?;

        // Add the presenter and co-host
        self.entities.upsert_entity(generate_presenter())?;
        self.entities.upsert_entity(generate_collector())?;

        Ok(())
    }

    pub fn all_entity_states(&self) -> Vec<Entity> {
        self.entities.get_all_entities().cloned().collect()
    }

    /// is the match over? True if there is 0-1 players left
    pub fn match_over(&self) -> bool {
        let player_count = self
            .entities
            .get_all_entities()
            .filter(|e| has_markers!(e, Player))
            .count();
        player_count <= 1
    }

    fn maybe_next_world_state(&mut self, entity_view: &EntityView, ctx: &ServerCtx) -> EntityWorld {
        let mut rng = rand::rng();
        let mut world_entity = entity_view
            .all()
            .find(|e| e.attributes.world.is_some())
            .expect("Expected world entity to exist")
            .clone();

        if rng.random_bool(0.005) {
            world_entity
                .attributes
                .world
                .as_mut()
                .unwrap()
                .update(&ctx.log_tx, &mut rng);
            self.entities.upsert_entity(world_entity.clone()).unwrap();
        }

        world_entity.attributes.world.unwrap()
    }
}

/// Event occuring during a tick
/// Is sent to clients so they can display the game in real-time
///
/// TIMING:
///
///  - StartOfTick
///  - (Processing happens on server)
///  - EntityChanges
///  - EndOfTick
#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TickEvent {
    /// A new tick has started
    StartOfTick { tick_id: TickId },

    /// A new tick has ended
    EndOfTick { tick_id: TickId },

    /// A new match just started
    /// (note: does not fire if resumed, only when completely new)
    StartOfMatch,

    /// The match ended
    EndOfMatch,

    /// Set of changes to entities during the last tick
    EntityChanges { changes: Vec<EntityManagerMutation> },
}
