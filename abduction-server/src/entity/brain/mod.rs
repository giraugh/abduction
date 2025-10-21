pub mod characteristic;
pub mod discussion;
pub mod focus;
pub mod motivator;
pub mod player_action;
pub mod signal;

use itertools::Itertools;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    seq::{IndexedRandom, IteratorRandom},
};
use tracing::warn;

use crate::{
    entity::{
        brain::{
            characteristic::{Characteristic, CharacteristicStrength},
            player_action::{PlayerAction, PlayerActionResult, PlayerActionSideEffect},
            signal::{Signal, SignalContext, SignalRef, WeightedPlayerActions},
        },
        Entity, EntityFood, EntityWaterSource,
    },
    event::{builder::GameEventBuilder, GameEventKind, GameEventTarget},
    has_markers,
    hex::AxialHexDirection,
    logs::{GameLog, GameLogBody},
    mtch::ActionCtx,
};
use focus::PlayerFocus;

impl Entity {
    /// Determine the next action to be taken by an entity
    /// Only applicable for players
    pub fn get_next_action<'a>(
        &'a self,
        event_signals: impl Iterator<Item = SignalRef<'a>>,
    ) -> PlayerAction {
        // Build the context for acting (WIP)
        let ctx = SignalContext {
            focus: self
                .attributes
                .focus
                .as_ref()
                .cloned()
                .unwrap_or(PlayerFocus::Unfocused),
        };

        // Collect signals
        let motivator_signals = self.attributes.motivators.as_signals();
        let signals = itertools::chain!(motivator_signals, event_signals);

        // Then resolve them into actions
        let mut actions = WeightedPlayerActions::default();
        signals.for_each(|signal| signal.act_on(&ctx, &mut actions));
        actions.sample(&mut rand::rng())
    }

    pub fn resolve_action(
        &mut self,
        action: PlayerAction,
        ctx: &mut ActionCtx,
    ) -> PlayerActionResult {
        match &action {
            PlayerAction::Nothing => {
                return PlayerActionResult::NoEffect;
            }

            PlayerAction::Sequential(sub_actions) => {
                for sub_action in sub_actions {
                    match self.resolve_action(sub_action.clone(), ctx) {
                        PlayerActionResult::SideEffect(side_effect) => {
                            return PlayerActionResult::SideEffect(side_effect)
                        }
                        PlayerActionResult::NoEffect => {
                            continue;
                        }
                        PlayerActionResult::Ok => {
                            break;
                        }
                    }
                }

                return PlayerActionResult::NoEffect;
            }

            PlayerAction::BumpMotivator(key) => {
                self.attributes.motivators.bump_key(*key);
                return PlayerActionResult::Ok;
            }

            PlayerAction::ReduceMotivator(key) => {
                self.attributes.motivators.reduce_key(*key);
                return PlayerActionResult::Ok;
            }

            PlayerAction::Discussion(discussion_action) => {
                return self.resolve_discussion_action(discussion_action, ctx)
            }

            PlayerAction::WakeUp => {
                match self.attributes.focus {
                    // If we are alreay sleeping, keep sleeping
                    Some(PlayerFocus::Sleeping { .. }) => {
                        self.attributes.focus = Some(PlayerFocus::Unfocused);

                        // Its very beneficial!
                        self.attributes.motivators.reduce_by::<motivator::Hurt>(0.2);

                        ctx.send_log(GameLog::entity(self, GameLogBody::EntityStopSleeping));
                    }
                    _ => return PlayerActionResult::NoEffect,
                }

                return PlayerActionResult::Ok;
            }

            PlayerAction::Sleep => {
                match self.attributes.focus {
                    // If we are alreay sleeping, keep sleeping
                    Some(PlayerFocus::Sleeping {
                        ref mut remaining_turns,
                    }) => {
                        // Wake up?
                        if *remaining_turns <= 1 {
                            self.attributes.focus = Some(PlayerFocus::Unfocused);

                            // Its very beneficial!
                            self.attributes.motivators.reduce_by::<motivator::Hurt>(0.2);

                            ctx.send_log(GameLog::entity(self, GameLogBody::EntityStopSleeping));
                        } else {
                            *remaining_turns -= 1;

                            // Get less tired
                            // (this way if we wake up part way, we are still groggy)
                            self.attributes
                                .motivators
                                .reduce_by::<motivator::Tiredness>(0.2);

                            ctx.send_log(GameLog::entity(self, GameLogBody::EntityKeepSleeping));
                        }
                    }

                    // Otherwise, start sleeping now
                    _ => {
                        self.attributes.focus = Some(PlayerFocus::Sleeping {
                            remaining_turns: 25,
                        });

                        ctx.send_log(GameLog::entity(self, GameLogBody::EntityStartSleeping));
                    }
                };

                return PlayerActionResult::Ok;
            }

            // Literally die
            PlayerAction::Death => {
                ctx.send_log(GameLog::entity(self, GameLogBody::EntityDeath));

                return PlayerActionResult::SideEffect(PlayerActionSideEffect::Death);
            }

            PlayerAction::MoveAwayFrom(log_body, markers) => {
                let mut rng = rand::rng();

                // Get entities at my location with that marker
                let avoid_entities = ctx
                    .all_entities
                    .iter()
                    .filter(|e| {
                        e.entity_id != self.entity_id
                            && e.attributes.hex.is_some()
                            && e.attributes.hex == self.attributes.hex
                    })
                    .filter(|e| markers.iter().any(|m| e.markers.contains(m)))
                    .collect_vec();

                // Is there at least one? If so choose one at random
                let Some(avoid_entity) = avoid_entities.choose(&mut rng) else {
                    return PlayerActionResult::NoEffect;
                };

                // Emit log
                ctx.send_log(GameLog::entity_pair(self, *avoid_entity, log_body.clone()));

                // Then move randomly
                let move_action = PlayerAction::all_movements()
                    .choose(&mut rng)
                    .unwrap()
                    .clone();
                return self.resolve_action(move_action, ctx);
            }

            PlayerAction::GoToAdjacent(log_body, markers) => {
                let mut rng = rand::rng();

                // Do we have a valid hex?
                let Some(my_hex) = self.attributes.hex else {
                    warn!("Attempted to search for adjacent entities but player has no hex");
                    return PlayerActionResult::NoEffect;
                };

                // Is the current hex such a hex?
                let current_hex_valid = ctx.all_entities.iter().any(|e| {
                    e.attributes.hex.is_some()
                        && e.attributes.hex == Some(my_hex)
                        && markers.iter().any(|m| e.markers.contains(m))
                });

                if current_hex_valid {
                    return PlayerActionResult::NoEffect;
                }

                // If not, pull all applicable adjacent entities
                let adj_entities = ctx
                    .all_entities
                    .iter()
                    .filter(|e| match e.attributes.hex {
                        None => false,
                        Some(hex) => hex.is_adjacent(my_hex),
                    })
                    .filter(|e| markers.iter().any(|m| e.markers.contains(m)))
                    .collect_vec();

                // If no relevant adjacent hexs, we cant do anything
                if adj_entities.is_empty() {
                    return PlayerActionResult::NoEffect;
                }

                // But if there is, choose one at random
                let chosen_entity = adj_entities.choose(&mut rng).unwrap();
                let hex = chosen_entity.attributes.hex.unwrap();
                let direction = AxialHexDirection::direction_to(my_hex, hex)
                    .expect("Cannot determine direction to adj hex");

                // Emit log
                ctx.send_log(GameLog::entity(self, log_body.clone()));

                // Travel towards that hex
                return self.resolve_action(PlayerAction::Move(direction), ctx);
            }

            // This is a little tricky lets be honest
            // I think I would just do easiest possible approach and move to the neighbour hex which reduces the distance the most
            PlayerAction::GoTowards(log_body, markers) => {
                // Do we have a valid hex?
                let Some(my_hex) = self.attributes.hex else {
                    warn!("Attempted to search for adjacent entities but player has no hex");
                    return PlayerActionResult::NoEffect;
                };

                // Is the current hex such a hex?
                let current_hex_valid = ctx.all_entities.iter().any(|e| {
                    e.attributes.hex.is_some()
                        && e.attributes.hex == Some(my_hex)
                        && markers.iter().any(|m| e.markers.contains(m))
                });

                if current_hex_valid {
                    return PlayerActionResult::NoEffect;
                }

                // If not, pull all applicable entities
                let target_entities = ctx
                    .all_entities
                    .iter()
                    .filter(|e| markers.iter().any(|m| e.markers.contains(m)))
                    .collect_vec();

                // If no relevant entities, we cant do anything
                if target_entities.is_empty() {
                    return PlayerActionResult::NoEffect;
                }

                // Now sort the target entities by distance
                let target_entity = target_entities
                    .iter()
                    .min_by_key(|e| e.attributes.hex.unwrap().dist_to(my_hex))
                    .unwrap();
                let target_hex = target_entity.attributes.hex.unwrap();

                // Next, find our adjacent hex which is closest to the target hex
                let adjacent_hex = my_hex
                    .neighbours()
                    .into_iter()
                    .filter(|h| h.within_bounds(ctx.config.world_radius as isize))
                    .min_by_key(|h| h.dist_to(target_hex))
                    .unwrap();

                // Emit log
                ctx.send_log(GameLog::entity(self, log_body.clone()));

                // And travel towards that
                let direction = AxialHexDirection::direction_to(my_hex, adjacent_hex).unwrap();
                return self.resolve_action(PlayerAction::Move(direction), ctx);
            }

            // Indicating a high motivator value
            PlayerAction::Bark(motivation, motivator) => {
                ctx.send_log(GameLog::entity(
                    self,
                    GameLogBody::EntityMotivatorBark {
                        motivation: *motivation,
                        motivator: *motivator,
                    },
                ));

                // This returns no effect so that the boredom is increased and to allow stacking barks + other actions w/ Sequential
                return PlayerActionResult::NoEffect;
            }

            PlayerAction::ConsumeFood {
                try_dubious,
                try_morally_wrong,
            } => {
                // Is there food at this location?
                let mut rng = rand::rng();
                let food_entities = ctx
                    .all_entities
                    .iter()
                    .filter(|e| {
                        e.attributes.hex.is_some() && e.attributes.hex == self.attributes.hex
                    })
                    .filter(|e| match e.attributes.food {
                        // Is it food at all?
                        None => false,

                        // Is it food but dubious?
                        Some(EntityFood {
                            poison,
                            morally_wrong,
                            ..
                        }) if poison > 0.0 => {
                            *try_dubious && (!morally_wrong || *try_morally_wrong)
                        }

                        // Good food
                        Some(EntityFood { .. }) => true,
                    });
                let Some(food_entity) = food_entities.choose(&mut rng) else {
                    return PlayerActionResult::NoEffect;
                };

                // if there is, eat it
                let food = food_entity.attributes.food.as_ref().unwrap();
                self.attributes
                    .motivators
                    .reduce_by::<motivator::Hunger>(food.sustenance.min(0.1));

                // is this morally wrong, hesitate for a second (send log before the eat log)
                if food.morally_wrong {
                    // TODO: maybe chance to bail based on a stat
                    ctx.send_log(GameLog::entity_pair(
                        self,
                        food_entity,
                        GameLogBody::EntityHesitateBeforeConsume,
                    ));
                }

                // emit log
                ctx.send_log(GameLog::entity_pair(
                    self,
                    food_entity,
                    GameLogBody::EntityConsume,
                ));

                // was it poisonous
                if food.sustenance < 0.0 {
                    self.attributes
                        .motivators
                        .bump_scaled::<motivator::Sickness>(food.sustenance);

                    ctx.send_log(GameLog::entity_pair(
                        self,
                        food_entity,
                        GameLogBody::EntityComplainAboutTaste,
                    ));
                }

                // Return side effect to remove the food
                return PlayerActionResult::SideEffect(PlayerActionSideEffect::RemoveOther(
                    food_entity.entity_id.clone(),
                ));
            }

            PlayerAction::DrinkFromWaterSource { try_dubious } => {
                // Is there food at this location?
                let mut rng = rand::rng();
                let water_source_entities = ctx
                    .all_entities
                    .iter()
                    .filter(|e| {
                        e.attributes.hex.is_some() && e.attributes.hex == self.attributes.hex
                    })
                    .filter(|e| match e.attributes.water_source {
                        // its dubious, are we okay with that?
                        Some(EntityWaterSource { poison }) if poison > 0.0 => *try_dubious,

                        // not dubious (fallthrough)
                        Some(EntityWaterSource { .. }) => true,

                        // its not a water source
                        None => false,
                    });

                // If no applicable water source, there's no effect
                let Some(water_source_entity) = water_source_entities.choose(&mut rng) else {
                    return PlayerActionResult::NoEffect;
                };

                // if there is, drink from it
                let water_source = water_source_entity
                    .attributes
                    .water_source
                    .as_ref()
                    .unwrap();

                // Fully clear thirst
                self.attributes.motivators.clear::<motivator::Thirst>();

                // Emit log
                ctx.send_log(GameLog::entity_pair(
                    self,
                    water_source_entity,
                    GameLogBody::EntityDrinkFrom,
                ));

                // Should we get sick?
                if water_source.poison > 0.0 {
                    self.attributes
                        .motivators
                        .bump_scaled::<motivator::Sickness>(2.0 * water_source.poison);
                    ctx.send_log(GameLog::entity_pair(
                        self,
                        water_source_entity,
                        GameLogBody::EntityComplainAboutTaste,
                    ));
                }

                return PlayerActionResult::Ok;
            }

            PlayerAction::TalkWithBeing { try_cannot_respond } => {
                let being_entities = ctx
                    .all_entities
                    .iter()
                    .filter(|e| {
                        e.attributes.hex.is_some()
                            && e.attributes.hex == self.attributes.hex
                            && e.entity_id != self.entity_id
                    })
                    .filter(
                        |e| match (has_markers!(e, CanTalk), has_markers!(e, Being)) {
                            // If its a human, always yes
                            (true, _) => true,

                            // Otherwise, if we are okay w/ non responders then yes
                            (_, true) => *try_cannot_respond,

                            // Otherwise don't talk with it
                            _ => false,
                        },
                    );

                // If no applicable being, there's no effect
                let mut rng = rand::rng();
                let Some(being_entity) = being_entities.choose(&mut rng) else {
                    return PlayerActionResult::NoEffect;
                };

                // Is there an established association relation?
                let association_bond_strength = self
                    .relations
                    .get_associate(&being_entity.entity_id)
                    .map(|associate| associate.bond)
                    .unwrap_or(0.0);

                // Log
                ctx.send_log(GameLog::entity_pair(
                    self,
                    being_entity,
                    GameLogBody::EntityGreet {
                        bond: association_bond_strength,
                    },
                ));

                // If they are unfriendly, this goes differently
                // NOTE: if they dont have motivators, we assume they are friendly (assuming that animals etc are friendly)
                // TODO: probably want to have a tag for beings that inverts this assumption (e.g Predator or something)
                let friendliness = being_entity.characteristic(Characteristic::Friendliness);
                if friendliness < CharacteristicStrength::Average {
                    // they ignore us
                    ctx.send_log(GameLog::entity_pair(
                        being_entity,
                        &self.entity_id,
                        GameLogBody::EntityIgnore,
                    ));

                    // And we like them less
                    self.relations
                        .decrease_associate_bond(&being_entity.entity_id);
                } else {
                    // Just them responding makes us like them
                    self.relations
                        .increase_associate_bond(&being_entity.entity_id);

                    // And if they can respond, we start a chat with them
                    if has_markers!(being_entity, CanTalk) {
                        // And we start talking to them
                        // Initial interest scales w/ bond but has a minimum
                        // (for simplicity our interest starts the same as theirs in the convo)
                        let max_interest = 20f32;
                        let interest = ((association_bond_strength * max_interest) as usize)
                            .clamp(2, max_interest as usize);

                        // Set our focus
                        self.attributes.focus = Some(PlayerFocus::Discussion {
                            with: being_entity.entity_id.clone(),
                            interest,
                        });

                        // TODO: maybe there's a strat here where we force them to do a "talk" action w/ us instead
                        return PlayerActionResult::SideEffect(PlayerActionSideEffect::SetFocus {
                            entity_id: being_entity.entity_id.clone(),
                            focus: PlayerFocus::Discussion {
                                with: self.entity_id.clone(),
                                interest,
                            },
                        });
                    }
                }
            }

            // Moving in a given hex direction
            PlayerAction::Move(hex_direction) => {
                let hex = self
                    .attributes
                    .hex
                    .as_mut()
                    .expect("Cannot move without hex attribute");
                let new_hex = *hex + (*hex_direction).into();
                if new_hex.within_bounds(ctx.config.world_radius as isize) {
                    // If succesfull, get thirsty and tired
                    self.attributes.motivators.bump::<motivator::Thirst>();
                    self.attributes
                        .motivators
                        .bump_scaled::<motivator::Tiredness>(0.3);

                    // And raise an event
                    GameEventBuilder::new()
                        .of_kind(GameEventKind::LeaveHex {
                            entity_id: self.entity_id.clone(),
                        })
                        .targets(GameEventTarget::Hex(*hex))
                        .with_sense(Characteristic::Vision, 0)
                        .with_sense(Characteristic::Hearing, 0)
                        .add(ctx);
                    GameEventBuilder::new()
                        .of_kind(GameEventKind::ArriveInHex {
                            entity_id: self.entity_id.clone(),
                        })
                        .targets(GameEventTarget::Hex(new_hex))
                        .with_sense(Characteristic::Vision, 0)
                        .with_sense(Characteristic::Hearing, 0)
                        .add(ctx);

                    // Actually move
                    *hex = new_hex;

                    // and a log
                    ctx.send_log(GameLog::entity(
                        self,
                        GameLogBody::EntityMovement { by: *hex_direction },
                    ));
                }
            }
        }

        PlayerActionResult::Ok
    }
}
