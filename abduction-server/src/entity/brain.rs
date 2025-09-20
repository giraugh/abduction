use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    seq::IteratorRandom,
};
use tokio::sync::broadcast;

use crate::{
    entity::{
        motivator::{self, MotivatorKey},
        Entity, EntityAsleep, EntityId, EntityMarker, EntityWaterSource,
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

    /// Try each action in the list until one works
    Sequential(Vec<PlayerAction>),

    /// Go to any adjacent hex with an entity that has the given marker
    GoToAdjacent(Vec<EntityMarker>),

    /// Die and be removed from the game
    Death,

    /// Exclaim about a high motivator of some kind
    Bark(f32, MotivatorKey),

    /// Move to a new hex
    Move(AxialHexDirection),

    /// Attempt to eat any food entity at current location
    ConsumeFood,

    /// Keep sleeping zzzzz
    Sleep,

    /// Drink from a water source at current location
    /// (including water that looks bad?)
    DrinkFromWaterSource { try_dubious: bool },

    /// Hurt by lack of food
    HungerPangs,

    /// Hurt by lack of water
    ThirstPangs,
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
                        self.attributes.asleep = Some(EntityAsleep { remaining_turns: 5 });

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

            PlayerAction::GoToAdjacent(markers) => {
                // TODO
                unimplemented!();
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

            PlayerAction::ConsumeFood => {
                // Is there food at this location?
                let mut rng = rand::rng();
                let food_entities = all_entities
                    .iter()
                    .filter(|e| {
                        e.attributes.hex.is_some() && e.attributes.hex == self.attributes.hex
                    })
                    .filter(|e| e.attributes.food.is_some());
                let Some(food_entity) = food_entities.choose(&mut rng) else {
                    return PlayerActionResult::NoEffect;
                };

                // if there is, eat it
                let food = food_entity.attributes.food.as_ref().unwrap();
                self.attributes
                    .motivators
                    .reduce_by::<motivator::Hunger>(food.sustenance);

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
                    self.attributes.motivators.bump::<motivator::Sickness>();

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

            PlayerAction::HungerPangs => {
                self.attributes.motivators.bump::<motivator::Hurt>();
            }

            PlayerAction::ThirstPangs => {
                self.attributes.motivators.bump::<motivator::Hurt>();
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
