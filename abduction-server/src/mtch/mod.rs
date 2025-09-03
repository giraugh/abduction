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

use anyhow::Context;
pub use config::*;

use itertools::Itertools;
use rand::{seq::IndexedRandom, Rng};
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::info;

use crate::{
    brain::PlayerActionSideEffect,
    entity::{
        motivator, Entity, EntityAttributes, EntityHazard, EntityManager, EntityManagerMutation,
        EntityMarker,
    },
    has_markers,
    hex::AxialHex,
    location::{generate_locations_for_world, Biome},
    logs::{GameLog, GameLogBody},
    player_gen::generate_player,
    Db, QubitCtx,
};

/// Id for a given match
/// (generated as a UUID but its just TEXT, can be anything...)
pub type MatchId = String;

/// An id identifying a specific tick
/// NOTE: Not scoped to a match but global for the server
/// NOTE: Tick ids are not unique and may overflow, just helps with debugging and testing
pub type TickId = usize;

pub struct MatchManager {
    pub match_config: MatchConfig,
    pub match_entities: EntityManager,
}

impl MatchManager {
    pub async fn load_match(match_config: MatchConfig, db: &Db) -> Self {
        // Create an entity manager and load the entities for the match
        let mut match_entities = EntityManager::new(&match_config.match_id);
        match_entities.load_entities(db).await;

        Self {
            match_config,
            match_entities,
        }
    }

    /// Load in a match configuration, generating any resources needed for the game
    ///
    /// This should only be done once per match, realistically - so prob do it when
    /// the config is created
    pub async fn initialise_new_match(&mut self, db: &Db) -> anyhow::Result<()> {
        // Now we initialise it...
        info!("Initialising match {}", &self.match_config.match_id);

        // Add all the unescaped players from the last game
        // In practice, this just means cloning the entity into the new match
        let mut existing_players = 0;
        if let Some(preceding_match_id) = &self.match_config.preceding_match_id {
            EntityManager::load_entities_from_match(preceding_match_id, db)
                .await
                .filter(|e| has_markers!(e, Player) && !has_markers!(e, Escaped))
                .for_each(|entity| {
                    existing_players += 1;
                    unimplemented!(); // TODO: actually copy across the entity
                })
        }

        // If we dont have enough players for the match configuration,
        // then generate and add more
        let player_count_to_gen = self.match_config.player_count - existing_players;
        for _ in 0..player_count_to_gen {
            let player_entity = generate_player().context("Generating player entity")?;
            self.match_entities.upsert_entity(player_entity)?;
        }

        // Generate a location entity in each hex
        for entity in
            generate_locations_for_world(self.match_config.world_radius as isize, Biome::Green)
        {
            self.match_entities.upsert_entity(entity)?;
        }

        // Put players in the desired locations
        // TODO

        // Put 3 random lava entities in the world (not at the origin though)
        let mut rng = rand::rng();
        for i in 0..3 {
            let position =
                AxialHex::random_in_bounds(&mut rng, self.match_config.world_radius as isize);
            self.match_entities.upsert_entity(Entity {
                entity_id: Entity::id(),
                name: format!("Lava Hazard {i}"),
                markers: vec![EntityMarker::Viewable],
                attributes: EntityAttributes {
                    hex: Some(position),
                    hazard: Some(EntityHazard { damage: 1 }),
                    ..Default::default()
                },
                ..Default::default()
            })?;
        }

        Ok(())
    }

    pub fn all_entity_states(&self) -> Vec<Entity> {
        self.match_entities.get_all_entities().cloned().collect()
    }

    /// Perform one game tick
    /// When a match is on, this is called every second or so to update the state of the world
    pub async fn perform_match_tick(&mut self, ctx: &QubitCtx) {
        // Lets just attempt to implement the main entity loop and see how we go I guess?
        // Rough plan is that each hex has one player action - the player who acted last acts now
        // This is encoded as the player with the highest `TicksWaited` attribute
        let players_in_hexes = self
            .match_entities
            .get_all_entities()
            .filter(|e| has_markers!(e, Player))
            .cloned() // :(
            .into_group_map_by(|e| e.attributes.hex.unwrap());
        for (_hex, players) in players_in_hexes {
            let mut rng = rand::rng();

            // World acting on players in this hex
            {
                if let Some(entity) = players.choose(&mut rng) {
                    let mut player = entity.clone();
                    self.resolve_world_effect_on_player(&mut player, &ctx.log_tx);
                    self.match_entities.upsert_entity(player).unwrap();
                }
            }

            // Player actions in this hex
            {
                if let Some(entity) = players.choose(&mut rng) {
                    // Get a new copy to preserve changes from above
                    // Skipping this step if they were removed
                    let Some(mut player) = self.match_entities.get_entity(&entity.entity_id) else {
                        continue;
                    };

                    // Go update it
                    match self.resolve_player_action(&mut player, &ctx.log_tx) {
                        Some(PlayerActionSideEffect::Death) => {
                            // Remove that player entity
                            self.match_entities
                                .remove_entity(&player.entity_id)
                                .unwrap();

                            // TODO: add a corpse
                            self.match_entities
                                .upsert_entity(Entity {
                                    entity_id: Entity::id(),
                                    markers: vec![EntityMarker::Viewable],
                                    name: format!("Corpse of {}", &player.name),
                                    attributes: EntityAttributes {
                                        hex: player.attributes.hex,
                                        corpse: Some(player.entity_id),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .unwrap();
                        }
                        None => {
                            self.match_entities.upsert_entity(player).unwrap();
                        }
                    }
                }
            }
        }

        // Flush changes to entities to the DB and to clients
        self.match_entities
            .flush_changes(&ctx.tick_tx, &ctx.db)
            .await
            .unwrap();
    }

    /// is the match over? True if there is 0-1 players left
    pub fn match_over(&self) -> bool {
        let player_count = self
            .match_entities
            .get_all_entities()
            .filter(|e| has_markers!(e, Player))
            .count();
        player_count <= 1
    }

    fn resolve_world_effect_on_player(
        &self,
        player: &mut Entity,
        log_tx: &broadcast::Sender<GameLog>,
    ) {
        let mut rng = rand::rng();

        // Is there a `hazard` entity at their hex?
        if player.attributes.hex.is_some() && rng.random_bool(0.7) {
            for entity in self
                .match_entities
                .get_all_entities()
                .filter(|e| e.attributes.hex == player.attributes.hex)
            {
                if let Some(hazard) = &entity.attributes.hazard {
                    for _ in 0..hazard.damage {
                        player.attributes.motivators.bump::<motivator::Hurt>();
                    }

                    log_tx
                        .send(GameLog::entity_pair(
                            entity,
                            player,
                            GameLogBody::HazardHurt,
                        ))
                        .unwrap();
                    break;
                }
            }

            return;
        }

        // Maybe they are just hungry/thirsty?
        if rng.random_bool(0.01) {
            // TODO: slowly tune this
            if rng.random_bool(0.5) {
                player.attributes.motivators.bump::<motivator::Hunger>();
            } else {
                player.attributes.motivators.bump::<motivator::Thirst>();
            }
        }
    }

    fn resolve_player_action(
        &mut self,
        player: &mut Entity,
        log_tx: &broadcast::Sender<GameLog>,
    ) -> Option<PlayerActionSideEffect> {
        let action = player.get_next_action();
        player.resolve_action(action, &self.match_config, log_tx)
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
