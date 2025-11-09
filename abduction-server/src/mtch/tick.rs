use itertools::Itertools;
use rand::{seq::IndexedRandom, Rng};
use tracing::warn;

use crate::{
    create_markers,
    entity::{
        brain::{
            actor_action::{ActorAction, ActorActionSideEffect},
            focus::ActorFocus,
            motivator,
        },
        gen::generate_corpse,
        snapshot::{EntitySnapshot, EntityView},
        world::{EntityWorld, TimeOfDay, WeatherKind},
        Entity, EntityAttributes, EntityHazard, EntityManager,
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
            world_state: &current_world_state,
            events_buffer: &mut events_buffer,
        };

        // Before any players act, the presenter/collector get to act
        if let Some(presenter_entity) = entities_view
            .all()
            .find(|e| e.attributes.presenter.is_some())
        {
            let mut rng = rand::rng();
            let events = action_ctx
                .events
                .get_event_signals_for_entity(presenter_entity);
            let action = presenter_entity.get_next_action_as_presenter(&action_ctx, events);
            Self::resolve_actor_action(
                &mut action_ctx,
                &mut self.entities,
                &mut rng,
                presenter_entity.clone(),
                action,
            );
        } else {
            warn!("No presenter.. uhh is present");
        };

        if let Some(collector_entity) = entities_view
            .all()
            .find(|e| e.attributes.collector.is_some())
        {
            let mut rng = rand::rng();
            let events = action_ctx
                .events
                .get_event_signals_for_entity(collector_entity);
            let action = collector_entity.get_next_action_as_collector(&action_ctx, events);
            Self::resolve_actor_action(
                &mut action_ctx,
                &mut self.entities,
                &mut rng,
                collector_entity.clone(),
                action,
            );
        } else {
            warn!("No collector is present");
        };

        // Lets just attempt to implement the main entity loop and see how we go I guess?
        // Rough plan is that each hex has one player action - the player who acted last acts now
        // This is encoded as the player with the highest `TicksWaited` attribute
        let players_in_hexes = entities_view
            .all()
            // cannot act if no hex
            .filter(|e| has_markers!(e, Player) && e.attributes.hex.is_some())
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
                    let Some(player) = self.entities.get_entity(&entity.entity_id) else {
                        continue;
                    };

                    // What are they going to do?
                    let events = action_ctx.events.get_event_signals_for_entity(&player);
                    let action = player.get_next_action(&action_ctx, events);

                    // Go update it
                    Self::resolve_actor_action(
                        &mut action_ctx,
                        &mut self.entities,
                        &mut rng,
                        player,
                        action,
                    );
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
        if current_world_state.weather.is_raining() {
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

        // Are they sheltering?
        // if so, some of the world stops acting on them
        let unfocused = matches!(player.attributes.focus, None | Some(ActorFocus::Unfocused));
        let sheltering = matches!(player.attributes.focus, Some(ActorFocus::Sheltering { .. }));

        // Is there a `hazard` entity at their hex?
        if player.attributes.hex.is_some() && rng.random_bool(0.7) && unfocused {
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
        if rng.random_bool(0.01) && unfocused {
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
            .world_state
            .time_of_day
            .current_temp_as_cold_proc_chance_scale();
        let cold_chance_scale_from_wind = ctx.world_state.weather.wind_proc_chance_scale();
        let cold_chance = cold_chance_scale_from_time * cold_chance_scale_from_wind * 0.2;
        if !sheltering && rng.random_bool(cold_chance as f64) {
            // TODO: prob need a way to find shelter or warm up huh
            player.attributes.motivators.bump::<motivator::Cold>();

            // Emit log
            ctx.send_log(GameLog::entity(
                player,
                GameLogBody::EntityColdBecauseOfTime,
            ));
        }

        // Warm up in the sun?
        if !sheltering && cold_chance_scale_from_time == 0.0 && rng.random_bool(0.05) {
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
        let rain_chance_scale = ctx.world_state.weather.rain_proc_chance_scale();
        if !sheltering && rng.random_bool((rain_chance_scale as f64) * 0.1) {
            // TODO: prob need a way to find shelter or warm up huh
            player.attributes.motivators.bump::<motivator::Saturation>();

            // Emit log
            ctx.send_log(GameLog::entity(
                player,
                GameLogBody::EntitySaturatedBecauseOfRain,
            ));
        }

        // Lightning strike?
        if !sheltering && matches!(ctx.world_state.weather, WeatherKind::LightningStorm) {
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
            || (ctx.world_state.time_of_day == TimeOfDay::Night && rng.random_bool(0.01))
        {
            player.attributes.motivators.bump::<motivator::Tiredness>();
        }
    }

    fn resolve_action_side_effect(
        entities: &mut EntityManager,
        rng: &mut impl rand::Rng,
        entity: Entity,
        side_effect: Option<ActorActionSideEffect>,
    ) {
        match side_effect {
            Some(ActorActionSideEffect::Death) => {
                // Remove this player entity
                entities.remove_entity(&entity.entity_id).unwrap();

                // Add a corpse
                entities
                    .upsert_entity(generate_corpse(rng, entity))
                    .unwrap();
            }
            Some(ActorActionSideEffect::RemoveOther(entity_id)) => {
                entities.remove_entity(&entity_id).unwrap();
                entities.upsert_entity(entity).unwrap();
            }
            Some(ActorActionSideEffect::BanishOther(entity_id)) => {
                // Remove the target entities hex
                let mut entity_to_banish = entities.get_entity(&entity_id).unwrap();
                entity_to_banish.attributes.hex = None;

                // Then update it, then update us as normal
                entities.upsert_entity(entity_to_banish).unwrap();
                entities.upsert_entity(entity).unwrap();
            }
            Some(ActorActionSideEffect::UnbanishOther(entity_id, hex)) => {
                // Set the target entities hex
                let mut entity_to_banish = entities.get_entity(&entity_id).unwrap();
                entity_to_banish.attributes.hex = Some(hex);

                // Then update it, then update us as normal
                entities.upsert_entity(entity_to_banish).unwrap();
                entities.upsert_entity(entity).unwrap();
            }
            Some(ActorActionSideEffect::SetFocus { entity_id, focus }) => {
                let mut other_entity = entities.get_entity(&entity_id).unwrap();
                other_entity.attributes.focus = Some(focus);
                entities.upsert_entity(other_entity).unwrap();
                entities.upsert_entity(entity).unwrap();
            }
            None => {
                entities.upsert_entity(entity).unwrap();
            }
        }
    }

    fn resolve_actor_action(
        ctx: &mut ActionCtx,
        entities: &mut EntityManager,
        rng: &mut impl rand::Rng,
        mut entity: Entity,
        action: ActorAction,
    ) {
        let result = entity.resolve_action(action, ctx);
        let side_effect = result.side_effect();

        // TODO: boredom bit used to be here but ehh, not sure if we want that anyway

        Self::resolve_action_side_effect(entities, rng, entity, side_effect);
    }
}
