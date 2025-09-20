use itertools::Itertools;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    seq::{IndexedRandom, IteratorRandom},
};
use tokio::sync::broadcast;
use tracing::warn;

use crate::{
    entity::{
        motivator::{self, MotivatorKey},
        Entity, EntityAsleep, EntityFood, EntityId, EntityMarker, EntityWaterSource,
    },
    hex::AxialHexDirection,
    logs::{GameLog, GameLogBody},
    mtch::MatchConfig,
};

#[derive(Clone, Debug)]
pub enum PlayerAction {
    /// No-op
    /// "<player> twiddles their thumbs" etc
    /// (This always causes the "NoEffect" result)
    Nothing,

    /// Increase some motivator by the sensitivity
    BumpMotivator(MotivatorKey),

    /// Decrease some motivator by the sensitivity
    ReduceMotivator(MotivatorKey),

    /// Try each action in the list until one works
    Sequential(Vec<PlayerAction>),

    /// Travel towards (the nearest?) hex which has an entity with any of the given markers
    /// NOTE: if already at such a location, this will do nothing (and cause NoEffect)
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    GoTowards(GameLogBody, Vec<EntityMarker>),

    /// Move to an adjacent hex where an entity resides with any of the given markers
    /// NOTE: if already at such a location, this will do nothing (and cause NoEffect)
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    GoToAdjacent(GameLogBody, Vec<EntityMarker>),

    /// Die and be removed from the game
    Death,

    /// Exclaim about a high motivator of some kind
    Bark(f32, MotivatorKey),

    /// Move to a new hex
    Move(AxialHexDirection),

    /// Attempt to eat any food entity at current location
    ConsumeFood {
        try_dubious: bool,
        try_morally_wrong: bool,
    },

    /// Keep sleeping zzzzz
    Sleep,

    /// Drink from a water source at current location
    /// (including water that looks bad?)
    DrinkFromWaterSource { try_dubious: bool },
}

#[derive(Clone, Debug)]
pub enum PlayerActionResult {
    /// Something that happens to the world as a result of player action
    SideEffect(PlayerActionSideEffect),

    /// Action had no effect
    /// (e.g try to eat food but there isnt any)
    NoEffect,

    /// Action succeeded (even if nothing happens)
    Ok,
}

impl PlayerActionResult {
    /// Get the side effect if there is one
    pub fn side_effect(self) -> Option<PlayerActionSideEffect> {
        match self {
            PlayerActionResult::SideEffect(player_action_side_effect) => {
                Some(player_action_side_effect)
            }
            PlayerActionResult::NoEffect => None,
            PlayerActionResult::Ok => None,
        }
    }
}

/// Something that happens to the world as a result of player action
#[derive(Clone, Debug)]
pub enum PlayerActionSideEffect {
    /// The player itself dies
    Death,

    /// Remove some other entity (e.g when eating food)
    RemoveOther(EntityId),
}

impl PlayerAction {
    #[inline(always)]
    pub const fn all_movements() -> &'static [Self] {
        use PlayerAction::*;
        &[
            Move(AxialHexDirection::East),
            Move(AxialHexDirection::NorthEast),
            Move(AxialHexDirection::SouthEast),
            Move(AxialHexDirection::West),
            Move(AxialHexDirection::NorthWest),
            Move(AxialHexDirection::SouthWest),
        ]
    }
}

impl Entity {
    /// Determine the next action to be taken by an entity
    /// Only applicable for players
    pub fn get_next_action(&self) -> PlayerAction {
        // Are we asleep? Then only valid action is sleeping
        // (This is decoupled from tiredness in case we have some other way of causing sleep)
        if self.attributes.asleep.is_some() {
            return PlayerAction::Sleep;
        }

        // Get the weighted actions from each motivator
        let mut action_weights = self.attributes.motivators.get_weighted_actions();

        // And add a no-op so its always an option
        action_weights.push((1, PlayerAction::Nothing));

        // TODO: remove impossible actions such as out-of-bounds movement here

        // Create a weighted distribution over the actions
        let (weights, actions): (Vec<_>, Vec<_>) = action_weights.into_iter().unzip();
        let dist = WeightedIndex::new(&weights).unwrap();

        // Sample the distribution
        let mut rng = rand::rng();
        actions[dist.sample(&mut rng)].clone()
    }

    pub fn resolve_action(
        &mut self,
        action: PlayerAction,
        all_entities: &Vec<Entity>,
        config: &MatchConfig,
        log_tx: &broadcast::Sender<GameLog>,
    ) -> PlayerActionResult {
        match &action {
            PlayerAction::Nothing => {
                return PlayerActionResult::NoEffect;
            }

            PlayerAction::Sequential(sub_actions) => {
                for sub_action in sub_actions {
                    match self.resolve_action(sub_action.clone(), all_entities, config, log_tx) {
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

            PlayerAction::Sleep => {
                match self.attributes.asleep.clone() {
                    // If we are alreay sleeping, keep sleeping
                    Some(asleep) => {
                        // Wake up?
                        if asleep.remaining_turns <= 1 {
                            self.attributes.asleep = None;

                            // Its very beneficial!
                            self.attributes.motivators.clear::<motivator::Tiredness>();
                            self.attributes.motivators.reduce_by::<motivator::Hurt>(0.2);

                            log_tx
                                .send(GameLog::entity(self, GameLogBody::EntityStopSleeping))
                                .unwrap();
                        } else {
                            self.attributes.asleep.as_mut().unwrap().remaining_turns -= 1;

                            log_tx
                                .send(GameLog::entity(self, GameLogBody::EntityKeepSleeping))
                                .unwrap();
                        }
                    }

                    // Otherwise, start sleeping now
                    None => {
                        // TODO: make this based on something ig
                        self.attributes.asleep = Some(EntityAsleep {
                            remaining_turns: 25,
                        });

                        log_tx
                            .send(GameLog::entity(self, GameLogBody::EntityStartSleeping))
                            .unwrap();
                    }
                };

                return PlayerActionResult::Ok;
            }

            // Literally die
            PlayerAction::Death => {
                log_tx
                    .send(GameLog::entity(self, GameLogBody::EntityDeath))
                    .unwrap();

                return PlayerActionResult::SideEffect(PlayerActionSideEffect::Death);
            }

            PlayerAction::GoToAdjacent(log_body, markers) => {
                let mut rng = rand::rng();

                // Do we have a valid hex?
                let Some(my_hex) = self.attributes.hex else {
                    warn!("Attempted to search for adjacent entities but player has no hex");
                    return PlayerActionResult::NoEffect;
                };

                // Is the current hex such a hex?
                let current_hex_valid = all_entities.iter().any(|e| {
                    e.attributes.hex.is_some()
                        && e.attributes.hex == Some(my_hex)
                        && markers.iter().any(|m| e.markers.contains(m))
                });

                if current_hex_valid {
                    return PlayerActionResult::NoEffect;
                }

                // If not, pull all applicable adjacent entities
                let adj_entities = all_entities
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
                log_tx
                    .send(GameLog::entity(self, log_body.clone()))
                    .unwrap();

                // Travel towards that hex
                return self.resolve_action(
                    PlayerAction::Move(direction),
                    all_entities,
                    config,
                    log_tx,
                );
            }

            // This is a little tricky lets be honest
            // I think I would just do easiest possible approach and move to the neighbour hex which reduces the distance the most
            PlayerAction::GoTowards(log_body, markers) => {
                let mut rng = rand::rng();

                // Do we have a valid hex?
                let Some(my_hex) = self.attributes.hex else {
                    warn!("Attempted to search for adjacent entities but player has no hex");
                    return PlayerActionResult::NoEffect;
                };

                // Is the current hex such a hex?
                let current_hex_valid = all_entities.iter().any(|e| {
                    e.attributes.hex.is_some()
                        && e.attributes.hex == Some(my_hex)
                        && markers.iter().any(|m| e.markers.contains(m))
                });

                if current_hex_valid {
                    return PlayerActionResult::NoEffect;
                }

                // If not, pull all applicable entities
                let target_entities = all_entities
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
                    .filter(|h| h.within_bounds(config.world_radius as isize))
                    .min_by_key(|h| h.dist_to(target_hex))
                    .unwrap();

                // Emit log
                log_tx
                    .send(GameLog::entity(self, log_body.clone()))
                    .unwrap();

                // And travel towards that
                let direction = AxialHexDirection::direction_to(my_hex, adjacent_hex).unwrap();
                return self.resolve_action(
                    PlayerAction::Move(direction),
                    all_entities,
                    config,
                    log_tx,
                );
            }

            // Indicating a high motivator value
            PlayerAction::Bark(motivation, motivator) => {
                log_tx
                    .send(GameLog::entity(
                        self,
                        GameLogBody::EntityMotivatorBark {
                            motivation: *motivation,
                            motivator: motivator.clone(),
                        },
                    ))
                    .unwrap();

                // This returns no effect so that the boredom is increased and to allow stacking barks + other actions w/ Sequential
                return PlayerActionResult::NoEffect;
            }

            PlayerAction::ConsumeFood {
                try_dubious,
                try_morally_wrong,
            } => {
                // Is there food at this location?
                let mut rng = rand::rng();
                let food_entities = all_entities
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
                    log_tx
                        .send(GameLog::entity_pair(
                            self,
                            food_entity,
                            GameLogBody::EntityHesitateBeforeConsume,
                        ))
                        .unwrap();
                }

                // emit log
                log_tx
                    .send(GameLog::entity_pair(
                        self,
                        food_entity,
                        GameLogBody::EntityConsume,
                    ))
                    .unwrap();

                // was it poisonous
                if food.sustenance < 0.0 {
                    self.attributes
                        .motivators
                        .bump_scaled::<motivator::Sickness>(food.sustenance);

                    log_tx
                        .send(GameLog::entity_pair(
                            self,
                            food_entity,
                            GameLogBody::EntityComplainAboutTaste,
                        ))
                        .unwrap();
                }

                // Return side effect to remove the food
                return PlayerActionResult::SideEffect(PlayerActionSideEffect::RemoveOther(
                    food_entity.entity_id.clone(),
                ));
            }

            PlayerAction::DrinkFromWaterSource { try_dubious } => {
                // Is there food at this location?
                let mut rng = rand::rng();
                let water_source_entities = all_entities
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
                log_tx
                    .send(GameLog::entity_pair(
                        self,
                        water_source_entity,
                        GameLogBody::EntityDrinkFrom,
                    ))
                    .unwrap();

                // Should we get sick?
                if water_source.poison > 0.0 {
                    self.attributes
                        .motivators
                        .bump_scaled::<motivator::Sickness>(2.0 * water_source.poison);
                    log_tx
                        .send(GameLog::entity_pair(
                            self,
                            water_source_entity,
                            GameLogBody::EntityComplainAboutTaste,
                        ))
                        .unwrap();
                }

                return PlayerActionResult::Ok;
            }

            // Moving in a given hex direction
            PlayerAction::Move(hex_direction) => {
                let hex = self
                    .attributes
                    .hex
                    .as_mut()
                    .expect("Cannot move without hex attribute");
                let new_hex = *hex + (*hex_direction).into();
                if new_hex.within_bounds(config.world_radius as isize) {
                    *hex = new_hex;

                    // If succesfull, get thirsty and tired
                    self.attributes.motivators.bump::<motivator::Thirst>();
                    self.attributes
                        .motivators
                        .bump_scaled::<motivator::Tiredness>(0.3);

                    log_tx
                        .send(GameLog::entity(
                            self,
                            GameLogBody::EntityMovement { by: *hex_direction },
                        ))
                        .unwrap();
                }
            }
        }

        PlayerActionResult::Ok
    }
}
