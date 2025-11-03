use itertools::Itertools;
use rand::{seq::IndexedRandom, Rng};

use crate::{
    create_markers,
    entity::{
        brain::{
            motivator,
            player_action::{PlayerActionResult, PlayerActionSideEffect},
        },
        gen::generate_corpse,
        snapshot::{EntitySnapshot, EntityView},
        world::{EntityWorld, TimeOfDay, WeatherKind},
        Entity, EntityAttributes, EntityHazard,
    },
    has_markers,
    hex::AxialHex,
    logs::{GameLog, GameLogBody},
    mtch::{ActionCtx, MatchManager},
    ServerCtx,
};

impl MatchManager {
    /// Perform one game tick
    /// When a match is on, this is called every second or so to update the state of the world
    pub async fn perform_match_tick(&mut self, ctx: &ServerCtx) {
        // Get all entities
        // this is our copy for performing this tick
        // NOTE: that entities wont be updated in here, so every entity kind of sees a frozen copy of the world
        //       until the next tick
        // let all_entities = self.entities.get_all_entities().cloned().collect_vec();
        let entity_snapshot =
            EntitySnapshot::new(self.entities.get_all_entities().cloned().collect_vec());
        let entities_view = entity_snapshot.view();

        // Perform world updates
        // i.e next time/weather
        let current_world_state = self.maybe_next_world_state(&entities_view, ctx);

        // Do global effects
        // (i.e that dont target specific players at random, just stuff everywhere)
        self.resolve_global_world_effects(&entities_view, &current_world_state, ctx);

        // Prepare a view for the events this tick
        // and a buffer of pending events
        let events = self.events.view();
        let mut events_buffer = Vec::new();

        // Build the context which we pass to each resolution method
        let mut action_ctx = ActionCtx {
            entities: &entities_view,
            events: &events,
            log_tx: &ctx.log_tx,
            config: &self.config,
            current_world_state: &current_world_state,
            events_buffer: &mut events_buffer,
        };

        // Lets just attempt to implement the main entity loop and see how we go I guess?
        // Rough plan is that each hex has one player action - the player who acted last acts now
        // This is encoded as the player with the highest `TicksWaited` attribute
        let players_in_hexes = entities_view
            .all()
            .filter(|e| has_markers!(e, Player))
            .cloned()
            .into_group_map_by(|e| e.attributes.hex.unwrap());
        for (_hex, players) in players_in_hexes {
            let mut rng = rand::rng();

            // World acting on players in this hex
            {
                if let Some(entity) = players.choose(&mut rng) {
                    let mut player = entity.clone();
                    self.resolve_world_effect_on_player(&mut player, &mut action_ctx);
                    self.entities.upsert_entity(player).unwrap();
                }
            }

            // Player actions in this hex
            {
                if let Some(entity) = players.choose(&mut rng) {
                    // Get a new copy to preserve changes from above
                    // Skipping this step if they were removed
                    let Some(mut player) = self.entities.get_entity(&entity.entity_id) else {
                        continue;
                    };

                    // Go update it
                    match self.resolve_player_action(&mut player, &mut action_ctx) {
                        Some(PlayerActionSideEffect::Death) => {
                            // Remove this player entity
                            self.entities.remove_entity(&player.entity_id).unwrap();

                            // Add a corpse
                            self.entities
                                .upsert_entity(generate_corpse(&mut rng, player))
                                .unwrap();
                        }
                        Some(PlayerActionSideEffect::RemoveOther(entity_id)) => {
                            self.entities.remove_entity(&entity_id).unwrap();
                            self.entities.upsert_entity(player).unwrap();
                        }
                        Some(PlayerActionSideEffect::BanishOther(entity_id)) => {
                            // Remove the target entities hex
                            let mut entity_to_banish =
                                self.entities.get_entity(&entity_id).unwrap();
                            entity_to_banish.attributes.hex = None;

                            // Then update it, then update us as normal
                            self.entities.upsert_entity(entity_to_banish).unwrap();
                            self.entities.upsert_entity(player).unwrap();
                        }
                        Some(PlayerActionSideEffect::UnbanishOther(entity_id, hex)) => {
                            // Set the target entities hex
                            let mut entity_to_banish =
                                self.entities.get_entity(&entity_id).unwrap();
                            entity_to_banish.attributes.hex = Some(hex);

                            // Then update it, then update us as normal
                            self.entities.upsert_entity(entity_to_banish).unwrap();
                            self.entities.upsert_entity(player).unwrap();
                        }
                        Some(PlayerActionSideEffect::SetFocus { entity_id, focus }) => {
                            let mut other_entity = self.entities.get_entity(&entity_id).unwrap();
                            other_entity.attributes.focus = Some(focus);
                            self.entities.upsert_entity(other_entity).unwrap();
                            self.entities.upsert_entity(player).unwrap();
                        }
                        None => {
                            self.entities.upsert_entity(player).unwrap();
                        }
                    }
                }
            }
        }

        // Flush changes to entities to the DB and to clients
        self.entities
            .flush_changes(&ctx.tick_tx, &ctx.db)
            .await
            .unwrap();

        // And empty out the event buffer
        // (by swapping it in)
        self.events.end_tick(events_buffer);
    }

    // Do global effects
    // i.e world updates that dont affect a given player, just spawn and move other stuff around
    // e.g spawn in hazards
    fn resolve_global_world_effects(
        &mut self,
        entities_view: &EntityView,
        current_world_state: &EntityWorld,
        ctx: &ServerCtx,
    ) {
        let mut rng = rand::rng();

        // Lightning starting fires
        if matches!(current_world_state.weather, WeatherKind::LightningStorm)
            && rng.random_bool(0.05)
        {
            let fire_entity = Entity {
                entity_id: Entity::id(),
                name: "Fire".into(),
                markers: create_markers!(Fire, Inspectable),
                attributes: EntityAttributes {
                    hex: Some(AxialHex::random_in_bounds(
                        &mut rng,
                        self.config.world_radius as isize,
                    )),
                    hazard: Some(EntityHazard { damage: 1 }),
                    ..Default::default()
                },
                ..Default::default()
            };

            ctx.log_tx
                .send(GameLog::entity(&fire_entity, GameLogBody::LightningStrike))
                .unwrap();

            self.entities.upsert_entity(fire_entity.clone()).unwrap();
        }

        // Fire spreading
        // TODO

        // Rain putting out fires
        if current_world_state.weather.rain_proc_chance_scale() > 0.0 {
            for entity in entities_view.all() {
                if has_markers!(entity, Fire) && rng.random_bool(0.05) {
                    self.entities.remove_entity(&entity.entity_id).unwrap();

                    // TODO: log this
                }
            }
        }
    }

    fn resolve_world_effect_on_player(&self, player: &mut Entity, ctx: &mut ActionCtx) {
        let mut rng = rand::rng();

        // Is there a `hazard` entity at their hex?
        if player.attributes.hex.is_some() && rng.random_bool(0.7) {
            for entity in self
                .entities
                .get_all_entities()
                .filter(|e| e.attributes.hex == player.attributes.hex)
            {
                if let Some(hazard) = &entity.attributes.hazard {
                    for _ in 0..hazard.damage {
                        player.attributes.motivators.bump::<motivator::Hurt>();
                    }

                    ctx.send_log(GameLog::entity_pair(
                        entity,
                        &player.entity_id,
                        GameLogBody::HazardHurt,
                    ));
                    break;
                }
            }

            return;
        }

        // Is there a water source at their location? They can fall in and get wet
        // TODO: maybe this is based on some kind of clumsiness stat?
        if rng.random_bool(0.01) {
            if let Some(water_source_entity) = self.entities.get_all_entities().find(|e| {
                e.attributes.water_source.is_some() && e.attributes.hex == player.attributes.hex
            }) {
                // Emit log
                ctx.send_log(GameLog::entity_pair(
                    player,
                    water_source_entity,
                    GameLogBody::EntityFellInWaterSource,
                ));

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
        let cold_chance_scale_from_time = ctx
            .current_world_state
            .time_of_day
            .current_temp_as_cold_proc_chance_scale();
        let cold_chance_scale_from_wind = ctx.current_world_state.weather.wind_proc_chance_scale();
        let cold_chance = cold_chance_scale_from_time * cold_chance_scale_from_wind * 0.2;
        if rng.random_bool(cold_chance as f64) {
            // TODO: prob need a way to find shelter or warm up huh
            player.attributes.motivators.bump::<motivator::Cold>();

            // Emit log
            ctx.send_log(GameLog::entity(
                player,
                GameLogBody::EntityColdBecauseOfTime,
            ));
        }

        // Warm up in the sun?
        if cold_chance_scale_from_time == 0.0 && rng.random_bool(0.05) {
            // Check we need to warm up
            if player
                .attributes
                .motivators
                .get_motivation::<motivator::Cold>()
                .unwrap_or(0.0)
                > 0.0
            {
                player
                    .attributes
                    .motivators
                    .reduce_by::<motivator::Cold>(0.3);

                ctx.send_log(GameLog::entity(
                    player,
                    GameLogBody::EntityWarmBecauseOfTime,
                ));
            }
        }

        // Is it raining?
        let rain_chance_scale = ctx.current_world_state.weather.rain_proc_chance_scale();
        if rng.random_bool((rain_chance_scale as f64) * 0.1) {
            // TODO: prob need a way to find shelter or warm up huh
            player.attributes.motivators.bump::<motivator::Saturation>();

            // Emit log
            ctx.send_log(GameLog::entity(
                player,
                GameLogBody::EntitySaturatedBecauseOfRain,
            ));
        }

        // Lightning strike?
        if matches!(ctx.current_world_state.weather, WeatherKind::LightningStorm) {
            // Quite rare to be direct hit
            if rng.random_bool(0.0005) {
                // Very damaging
                player
                    .attributes
                    .motivators
                    .bump_scaled::<motivator::Hurt>(20.0);

                // Emit log
                ctx.send_log(GameLog::entity(player, GameLogBody::EntityHitByLightning))
            }
        }

        // Or tired?
        // (more at night)
        if rng.random_bool(0.005)
            || (ctx.current_world_state.time_of_day == TimeOfDay::Night && rng.random_bool(0.01))
        {
            player.attributes.motivators.bump::<motivator::Tiredness>();
        }
    }

    fn resolve_player_action(
        &self,
        player: &mut Entity,
        ctx: &mut ActionCtx,
    ) -> Option<PlayerActionSideEffect> {
        let events = ctx.events.get_event_signals_for_entity(player);
        let action = player.get_next_action(ctx, events);
        let result = player.resolve_action(action, ctx);

        // TODO: perhaps if the resolved action had no effect, I could let them try again N times?

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
