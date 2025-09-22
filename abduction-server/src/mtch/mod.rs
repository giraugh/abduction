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
    create_markers,
    entity::{
        brain::{PlayerActionResult, PlayerActionSideEffect},
        motivator,
        world::{EntityWorld, WeatherKind},
        Entity, EntityAttributes, EntityFood, EntityHazard, EntityManager, EntityManagerMutation,
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
        let player_count_to_gen = self.match_config.player_count - existing_players;
        for _ in 0..player_count_to_gen {
            let player_entity = generate_player().context("Generating player entity")?;
            self.match_entities.upsert_entity(player_entity)?;
        }

        // Generate a location entity in each hex
        let mut rng = rand::rng();
        for entity in
            generate_locations_for_world(self.match_config.world_radius as isize, Biome::Green)
        {
            // Create the location
            self.match_entities.upsert_entity(entity.clone())?;

            // Generate some amount of props in each hex
            let hex = entity.attributes.hex.as_ref().unwrap();
            let location_kind = entity.attributes.location.as_ref().unwrap().location_kind;
            let prop_count = rng.random_range(0..5);
            let prop_generators = location_kind.prop_generators();

            // Generate required entities for location type
            for required_generator in &prop_generators.required {
                let mut entity = required_generator.generate(&mut rng);
                // Set its location and insert it
                entity.attributes.hex = Some(*hex);
                self.match_entities.upsert_entity(entity)?;
            }

            // Generate a few from the optional generators
            if !prop_generators.optional.is_empty() {
                for _ in 0..prop_count {
                    let entity = prop_generators.generate_optional_at(*hex, &mut rng);
                    self.match_entities.upsert_entity(entity)?;
                }
            }
        }

        // Establish the current state of the world
        self.match_entities.upsert_entity(Entity {
            entity_id: Entity::id(),
            name: "World".into(),
            attributes: EntityAttributes {
                world: Some(EntityWorld::default()),
                ..Default::default()
            },
            ..Default::default()
        })?;

        // Put players in the desired locations
        // TODO

        Ok(())
    }

    pub fn all_entity_states(&self) -> Vec<Entity> {
        self.match_entities.get_all_entities().cloned().collect()
    }

    /// Perform one game tick
    /// When a match is on, this is called every second or so to update the state of the world
    pub async fn perform_match_tick(&mut self, ctx: &QubitCtx) {
        // Get all entities
        // this is our copy for performing this tick
        // NOTE: that entities wont be updated in here, so every entity kind of sees a frozen copy of the world
        //       until the next tick
        let all_entities = self
            .match_entities
            .get_all_entities()
            .cloned()
            .collect_vec();

        // Perform world updates
        // i.e next time/weather
        let current_world_state = self.maybe_next_world_state(&all_entities, ctx);

        // Do global effects
        // (i.e that dont target specific players at random, just stuff everywhere)
        self.resolve_global_world_effects(&all_entities, &current_world_state, ctx);

        // Lets just attempt to implement the main entity loop and see how we go I guess?
        // Rough plan is that each hex has one player action - the player who acted last acts now
        // This is encoded as the player with the highest `TicksWaited` attribute
        let players_in_hexes = all_entities
            .clone()
            .into_iter()
            .filter(|e| has_markers!(e, Player))
            .into_group_map_by(|e| e.attributes.hex.unwrap());
        for (_hex, players) in players_in_hexes {
            let mut rng = rand::rng();

            // World acting on players in this hex
            {
                if let Some(entity) = players.choose(&mut rng) {
                    let mut player = entity.clone();
                    self.resolve_world_effect_on_player(
                        &mut player,
                        &current_world_state,
                        &ctx.log_tx,
                    );
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
                    match self.resolve_player_action(&mut player, &all_entities, &ctx.log_tx) {
                        Some(PlayerActionSideEffect::Death) => {
                            // Remove that player entity
                            self.match_entities
                                .remove_entity(&player.entity_id)
                                .unwrap();

                            // Add a corpse
                            self.match_entities
                                .upsert_entity(Entity {
                                    entity_id: Entity::id(),
                                    markers: vec![EntityMarker::DefaultInspectable],
                                    name: format!("Corpse of {}", &player.name),
                                    attributes: EntityAttributes {
                                        hex: player.attributes.hex,
                                        corpse: Some(player.entity_id),
                                        food: Some(EntityFood {
                                            morally_wrong: true,
                                            ..EntityFood::dubious(&mut rng)
                                        }),
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .unwrap();
                        }
                        Some(PlayerActionSideEffect::RemoveOther(entity_id)) => {
                            self.match_entities.remove_entity(&entity_id).unwrap();
                            self.match_entities.upsert_entity(player).unwrap();
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

    fn maybe_next_world_state(&mut self, all_entities: &[Entity], ctx: &QubitCtx) -> EntityWorld {
        let mut rng = rand::rng();
        let mut world_entity = all_entities
            .iter()
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
            self.match_entities
                .upsert_entity(world_entity.clone())
                .unwrap();
        }

        world_entity.attributes.world.unwrap()
    }

    // Do global effects
    // i.e world updates that dont affect a given player, just spawn and move other stuff around
    // e.g spawn in hazards
    fn resolve_global_world_effects(
        &mut self,
        all_entities: &[Entity],
        current_world_state: &EntityWorld,
        ctx: &QubitCtx,
    ) {
        let mut rng = rand::rng();

        // Lightning starting fires
        if matches!(current_world_state.weather, WeatherKind::LightningStorm)
            && rng.random_bool(0.05)
        {
            self.match_entities
                .upsert_entity(Entity {
                    entity_id: Entity::id(),
                    name: "Fire".into(),
                    markers: create_markers!(Fire, DefaultInspectable),
                    attributes: EntityAttributes {
                        hex: Some(AxialHex::random_in_bounds(
                            &mut rng,
                            self.match_config.world_radius as isize,
                        )),
                        hazard: Some(EntityHazard { damage: 1 }),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .unwrap();

            // TODO: log this
        }

        // Fire spreading
        // TODO

        // Rain putting out fires
        if current_world_state.weather.rain_proc_chance_scale() > 0.0 {
            for entity in all_entities {
                if has_markers!(entity, Fire) && rng.random_bool(0.05) {
                    self.match_entities
                        .remove_entity(&entity.entity_id)
                        .unwrap();

                    // TODO: log this
                }
            }
        }
    }

    fn resolve_world_effect_on_player(
        &self,
        player: &mut Entity,
        current_world_state: &EntityWorld,
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

        // Is there a water source at their location? They can fall in and get wet
        // TODO: maybe this is based on some kind of clumsiness stat?
        if rng.random_bool(0.01) {
            if let Some(water_source_entity) = self.match_entities.get_all_entities().find(|e| {
                e.attributes.water_source.is_some() && e.attributes.hex == player.attributes.hex
            }) {
                // Emit log
                log_tx
                    .send(GameLog::entity_pair(
                        player,
                        water_source_entity,
                        GameLogBody::EntityFellInWaterSource,
                    ))
                    .unwrap();

                // Up saturation
                player
                    .attributes
                    .motivators
                    .bump_scaled::<motivator::Saturation>(2.0);
            }
        }

        // Maybe they are just hungry/thirsty?
        if rng.random_bool(0.02) {
            // TODO: slowly tune this
            if rng.random_bool(0.5) {
                player.attributes.motivators.bump::<motivator::Hunger>();
            } else {
                player.attributes.motivators.bump::<motivator::Thirst>();
            }
        }

        // Is it cold?
        let cold_chance_scale_from_time = current_world_state
            .time_of_day
            .current_temp_as_cold_proc_chance_scale();
        let cold_chance_scale_from_wind = current_world_state.weather.wind_proc_chance_scale();
        let cold_chance = cold_chance_scale_from_time * cold_chance_scale_from_wind;
        if rng.random_bool((cold_chance as f64) * 0.1) {
            // TODO: prob need a way to find shelter or warm up huh
            player.attributes.motivators.bump::<motivator::Cold>();

            // Emit log
            log_tx
                .send(GameLog::entity(
                    player,
                    GameLogBody::EntityColdBecauseOfTime,
                ))
                .unwrap();
        }

        // Is it raining?
        let rain_chance_scale = current_world_state.weather.rain_proc_chance_scale();
        if rng.random_bool((rain_chance_scale as f64) * 0.1) {
            // TODO: prob need a way to find shelter or warm up huh
            player.attributes.motivators.bump::<motivator::Saturation>();

            // Emit log
            log_tx
                .send(GameLog::entity(
                    player,
                    GameLogBody::EntitySaturatedBecauseOfRain,
                ))
                .unwrap();
        }

        // Lightning strike?
        if matches!(current_world_state.weather, WeatherKind::LightningStorm) {
            // Quite rare to be direct hit
            if rng.random_bool(0.0005) {
                // Very damaging
                player
                    .attributes
                    .motivators
                    .bump_scaled::<motivator::Hurt>(5.0);

                // Emit log
                log_tx
                    .send(GameLog::entity(player, GameLogBody::EntityHitByLightning))
                    .unwrap();
            }
        }

        // Or tired?
        // TODO: more at night
        if rng.random_bool(0.005) {
            player.attributes.motivators.bump::<motivator::Tiredness>();
        }
    }

    fn resolve_player_action(
        &mut self,
        player: &mut Entity,
        all_entities: &Vec<Entity>,
        log_tx: &broadcast::Sender<GameLog>,
    ) -> Option<PlayerActionSideEffect> {
        let action = player.get_next_action();
        let result = player.resolve_action(action, all_entities, &self.match_config, log_tx);

        // If the last thing they did had no result, they get bored
        if matches!(result, PlayerActionResult::NoEffect) {
            player
                .attributes
                .motivators
                .bump_scaled::<motivator::Boredom>(2.0); // mostly temp for dev
        } else {
            player.attributes.motivators.clear::<motivator::Boredom>();
        }

        result.side_effect()
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
