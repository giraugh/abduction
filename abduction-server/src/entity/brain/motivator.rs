#![allow(clippy::single_match)]

use anyhow::anyhow;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::{collections::HashMap, fmt, str::FromStr};
use tracing::warn;

use super::{
    player_action::PlayerAction,
    signal::{Signal, SignalContext, SignalRef},
};
use crate::{
    create_markers,
    entity::brain::{
        discussion::DiscussionAction, focus::PlayerFocus, signal::WeightedPlayerActions,
    },
    logs::GameLogBody,
};

// Thanks GPT I guess
macro_rules! seq {
    ($($x:expr),*; ..$y:expr $(,)? ) => {
        {
            let mut v = vec![ $($x),* ];
            v.extend_from_slice($y);
            v
        }
    };

    ($($x:expr),* $(,)? ) => {
        vec![ $($x),* ]
    };
}

/// An attribute which "motivates" behaviour for an entity
/// primarily represented by a single 0-1 float
/// entity can react differently to motivators, so they have a
/// sensitity scalar which attenuates incoming "motiviation"
///
/// e.g if sensitivity is 0 for hunger -> that entity does not need to eat
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(from = "MotivatorDataTuple", into = "MotivatorDataTuple")]
#[qubit::ts]
pub struct MotivatorData {
    /// 0-1 motivation
    motivation: f32,
    /// 0-1 sensitivity
    sensitivity: f32,
}

#[derive(Serialize, Deserialize)]
#[qubit::ts]
struct MotivatorDataTuple(f32, f32);

impl From<MotivatorData> for MotivatorDataTuple {
    fn from(value: MotivatorData) -> Self {
        Self(value.motivation, value.sensitivity)
    }
}

impl From<MotivatorDataTuple> for MotivatorData {
    fn from(value: MotivatorDataTuple) -> Self {
        Self {
            motivation: value.0,
            sensitivity: value.1,
        }
    }
}

/// How to initialise a motivator?
pub enum MotivatorInit {
    /// Motivation at 0 (sensitivity still random)
    Zero,

    /// Completely random 0-1 motivation
    #[allow(unused)]
    Random,

    /// Random 0-1 but split into N levels
    #[allow(unused)]
    RandomDiscrete(usize),
}

pub trait Motivator {
    const TABLE_KEY: MotivatorKey;
    const INIT: MotivatorInit;

    fn init() -> MotivatorData {
        let mut rng = rng();
        let sensitivity = rng.random_range(0.01..=0.1);
        match Self::INIT {
            MotivatorInit::Zero => MotivatorData {
                sensitivity,
                motivation: 0.0,
            },
            MotivatorInit::Random => MotivatorData {
                sensitivity,
                motivation: rng.random_range(0.0..=1.0),
            },
            MotivatorInit::RandomDiscrete(n) => MotivatorData {
                sensitivity,
                motivation: (rng.random_range(0.0..=1.0) * n as f32).round() / (n as f32),
            },
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[qubit::ts]
pub struct MotivatorTable(HashMap<MotivatorKey, MotivatorData>);

impl MotivatorTable {
    pub fn insert<K: Motivator>(&mut self, data: MotivatorData) {
        self.0.insert(K::TABLE_KEY, data);
    }

    pub fn get_motivation<K: Motivator>(&self) -> Option<f32> {
        self.0.get(&K::TABLE_KEY).map(|m| m.motivation)
    }

    /// Increment a motivator by the sensitivity
    pub fn bump<K: Motivator>(&mut self) {
        self.bump_key(K::TABLE_KEY);
    }

    /// Increment a motivator by the sensitivity
    pub fn bump_key(&mut self, key: MotivatorKey) {
        if let Some(data) = self.0.get_mut(&key) {
            data.motivation = (data.motivation + data.sensitivity).clamp(0.0, 1.0);
        } else {
            warn!("Entity is missing motivator data for {:?}", key);
        }
    }

    /// Increment a motivator by the sensitivity (with some scaling factor)
    pub fn bump_scaled<K: Motivator>(&mut self, scale: f32) {
        if let Some(data) = self.0.get_mut(&K::TABLE_KEY) {
            data.motivation = (data.motivation + data.sensitivity * scale).clamp(0.0, 1.0);
        } else {
            warn!("Entity is missing motivator data for {:?}", K::TABLE_KEY);
        }
    }

    /// Clear out a motivation, setting it back to 0
    pub fn clear<K: Motivator>(&mut self) {
        if let Some(data) = self.0.get_mut(&K::TABLE_KEY) {
            data.motivation = 0.0;
        } else {
            warn!("Entity is missing motivator data for {:?}", K::TABLE_KEY);
        }
    }

    /// Decrement a motivator by the sensitivity
    #[allow(unused)]
    pub fn reduce<K: Motivator>(&mut self) {
        self.reduce_key(K::TABLE_KEY);
    }

    /// Decrement a motivator, specified by key, by the sensitivity
    pub fn reduce_key(&mut self, key: MotivatorKey) {
        if let Some(data) = self.0.get_mut(&key) {
            data.motivation = (data.motivation - data.sensitivity).clamp(0.0, 1.0);
        } else {
            warn!("Entity is missing motivator data for {:?}", key);
        }
    }

    /// Decrement a motivator by the specified amount
    pub fn reduce_by<K: Motivator>(&mut self, by: f32) {
        if let Some(data) = self.0.get_mut(&K::TABLE_KEY) {
            data.motivation = (data.motivation - by).clamp(0.0, 1.0);
        } else {
            warn!("Entity is missing motivator data for {:?}", K::TABLE_KEY);
        }
    }
}

macro_rules! declare_motivators {
    ({ $($keys:ident : $init: expr),* }) => {
        /// Declare the possible motivator keys
        #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
        #[serde(rename_all = "snake_case")]
        #[qubit::ts]
        pub enum MotivatorKey {
            $($keys,)*
        }

        // Then declare each struct
        $(
            #[derive(Debug)]
            pub struct $keys(MotivatorData);

            impl $keys {
                pub fn motivation(&self) -> f32 {
                    self.0.motivation
                }
            }

            impl Motivator for $keys {
                const TABLE_KEY: MotivatorKey = MotivatorKey::$keys;
                const INIT: MotivatorInit = $init;
            }
        )*

        // And create a method which gets a random state for each motivator
        impl MotivatorTable {
            pub fn initialise() -> Self {
                let mut table = Self::default();
                $(table.insert::<$keys>($keys::init());)*
                table
            }

            pub fn as_signals(&self) -> impl Iterator<Item = SignalRef> {
                let mut signals: Vec<SignalRef> = Vec::new();

                $({
                    if let Some(behaviour_data) = self.0.get(&$keys::TABLE_KEY) {
                        let signal = $keys(behaviour_data.clone());
                        signals.push(SignalRef::boxed(signal));
                    } else {
                        warn!("Entity is missing motivator data for {:?}", $keys::TABLE_KEY);
                    }
                })*

                signals.into_iter()
            }
        }
    }
}

declare_motivators!({
    Hunger: MotivatorInit::Zero,
    Thirst: MotivatorInit::Zero,
    Boredom: MotivatorInit::Zero,
    Hurt: MotivatorInit::Zero,
    Sickness: MotivatorInit::Zero,
    Tiredness: MotivatorInit::Zero,
    Saturation: MotivatorInit::Zero,
    Cold: MotivatorInit::Zero,
    Sadness: MotivatorInit::Zero
});

impl Signal for Hunger {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                // The generic plan for finding food
                let seek_food_plan: &[PlayerAction] = &[
                    PlayerAction::GoToAdjacent(
                        GameLogBody::EntityGoToAdjacentLush,
                        create_markers!(LushLocation),
                    ),
                    PlayerAction::Bark(self.motivation(), MotivatorKey::Hunger),
                ];

                // Eat food if we have it, maybe try finding some
                if self.motivation() > 0.3 {
                    actions.add(
                        if self.motivation() > 0.7 { 30 } else { 10 },
                        PlayerAction::Sequential(seq![
                            PlayerAction::ConsumeNearbyFood { try_dubious: false, try_morally_wrong: false },
                            PlayerAction::RetrieveInventoryFood;
                            ..seek_food_plan,
                        ]),
                    );
                }

                // Bit more desperate, eat bad food if thats all there is
                if self.motivation() > 0.6 {
                    actions.add(
                                if self.motivation() > 0.7 { 30 } else { 10 },
                                PlayerAction::Sequential(seq![
                                    PlayerAction::ConsumeNearbyFood { try_dubious: false, try_morally_wrong: false },
                                    PlayerAction::RetrieveInventoryFood,
                                    PlayerAction::ConsumeNearbyFood { try_dubious: true, try_morally_wrong: false };
                                    ..seek_food_plan,
                                ]),
                            );
                }

                // if extremely hungry, we'll try absolutely desperate things
                if self.motivation() > 0.9 {
                    actions.add(
                        10,
                        PlayerAction::ConsumeNearbyFood {
                            try_dubious: true,
                            try_morally_wrong: true,
                        },
                    );
                    // actions.add(10, PlayerAction::CannibalizeSelf);
                }

                if self.motivation() > 0.9 {
                    actions.add(20, PlayerAction::BumpMotivator(MotivatorKey::Hurt));
                }
            }
            PlayerFocus::Discussion { .. } => {
                // Stop talking, im hungry!
                if self.motivation() > 0.6 {
                    actions.add(
                        if self.motivation() > 0.7 { 30 } else { 10 },
                        PlayerAction::Sequential(seq![
                            PlayerAction::Bark(self.motivation(), MotivatorKey::Hunger),
                            PlayerAction::Discussion(DiscussionAction::LoseInterest),
                        ]),
                    );
                }
            }
            _ => {}
        }
    }
}

impl Signal for Thirst {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                // The generic plan for finding water
                let seek_water_plan: &[PlayerAction] = &[
                    PlayerAction::GoToAdjacent(
                        GameLogBody::EntityGoToAdjacentLush,
                        create_markers!(LushLocation),
                    ),
                    PlayerAction::GoTowards(
                        GameLogBody::EntityGoDownhill,
                        create_markers!(LowLyingLocation),
                    ),
                    PlayerAction::Bark(self.motivation(), MotivatorKey::Thirst),
                ];

                // Little bit thirsty, start trying to get water
                if self.motivation() > 0.4 {
                    actions.add(
                        20,
                        PlayerAction::Sequential(seq![
                            PlayerAction::DrinkFromWaterSource { try_dubious: false }; // Only go in for safe water
                            ..seek_water_plan,
                        ]),
                    );
                }

                // Urgent Drinking! Drink whatever we have available
                if self.motivation() > 0.7 {
                    actions.add(
                        30,
                        PlayerAction::Sequential(seq![
                            PlayerAction::DrinkFromWaterSource { try_dubious: false },
                            PlayerAction::DrinkFromWaterSource { try_dubious: true };
                            ..seek_water_plan,
                        ]),
                    );
                }

                if self.motivation() > 0.9 {
                    actions.add(20, PlayerAction::BumpMotivator(MotivatorKey::Hurt));
                }
            }
            PlayerFocus::Discussion { .. } => {
                // Stop talking, im thirsty!
                if self.motivation() > 0.6 {
                    actions.add(
                        if self.motivation() > 0.7 { 30 } else { 10 },
                        PlayerAction::Sequential(seq![
                            PlayerAction::Bark(self.motivation(), MotivatorKey::Hunger),
                            PlayerAction::Discussion(DiscussionAction::LoseInterest),
                        ]),
                    );
                }
            }
            _ => {}
        }
    }
}

impl Signal for Boredom {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                if self.motivation() > 0.5 {
                    actions.add(
                        2,
                        PlayerAction::Bark(self.motivation(), MotivatorKey::Boredom),
                    );
                }

                // If bored enough, do a random movement
                if self.motivation() > 0.7 {
                    actions.extend(
                        PlayerAction::all_movements()
                            .iter()
                            .cloned()
                            .map(|action| (25, action)),
                    );
                }
            }
            _ => {}
        }
    }
}

impl Signal for Hurt {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                if self.motivation() > 0.5 {
                    actions.add(5, PlayerAction::Bark(self.motivation(), MotivatorKey::Hurt));
                    actions.add(2, PlayerAction::BumpMotivator(MotivatorKey::Sadness));
                }
            }
            _ => {}
        }

        // Can die regardless of focus
        // If fully "motivated" then die
        if self.motivation() >= 0.99 {
            actions.add(1000, PlayerAction::Death);
        }
    }
}

impl Signal for Sickness {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                if self.motivation() > 0.0 {
                    // Its possible for it to randomly get worse or better
                    // slightly favouring getting better
                    actions.add(8, PlayerAction::ReduceMotivator(MotivatorKey::Sickness));
                    actions.add(5, PlayerAction::BumpMotivator(MotivatorKey::Sickness));
                }

                if self.motivation() > 0.5 {
                    actions.add(
                        10,
                        PlayerAction::Bark(self.motivation(), MotivatorKey::Sickness),
                    );
                }

                if self.motivation() > 0.8 {
                    actions.add(
                        10,
                        PlayerAction::Bark(self.motivation(), MotivatorKey::Sickness),
                    );
                    actions.add(10, PlayerAction::BumpMotivator(MotivatorKey::Hurt));
                }
            }
            _ => {}
        }
    }
}

impl Signal for Tiredness {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                if self.motivation() > 0.7 {
                    actions.add(
                        10,
                        PlayerAction::Bark(self.motivation(), MotivatorKey::Tiredness),
                    );
                }

                if self.motivation() > 0.8 {
                    actions.add(
                        20,
                        PlayerAction::Bark(self.motivation(), MotivatorKey::Tiredness),
                    );
                    actions.add(
                        if self.motivation() > 0.95 { 50 } else { 10 },
                        PlayerAction::Sleep,
                    );
                }
            }
            _ => {}
        }
    }
}

// Is wet for whatever reason
impl Signal for Saturation {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                if self.motivation() > 0.0 {
                    // Complain about being wet
                    actions.add(
                        15,
                        PlayerAction::Bark(self.motivation(), MotivatorKey::Saturation),
                    );

                    // Just slowly become dry
                    actions.add(5, PlayerAction::ReduceMotivator(MotivatorKey::Saturation));
                }

                if self.motivation() > 0.1 {
                    // Get more cold
                    actions.add(10, PlayerAction::BumpMotivator(MotivatorKey::Cold));

                    // Maybe get sick
                    actions.add(
                        if self.motivation() > 0.5 { 10 } else { 20 },
                        PlayerAction::BumpMotivator(MotivatorKey::Sickness),
                    );
                }

                // If raining, go seek shelter
                if self.motivation() > 0.1 && ctx.world_state.weather.is_raining() {
                    actions.add(
                        10,
                        PlayerAction::Sequential(vec![
                            PlayerAction::TakeShelter,
                            PlayerAction::SeekShelter,
                            PlayerAction::Bark(self.motivation(), MotivatorKey::Saturation),
                        ]),
                    );
                }
            }
            _ => {}
        }
    }
}

impl Signal for Cold {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                // TODO: more intelligent plans like finding shelter etc

                // If cold, go seek shelter
                if self.motivation() > 0.4 {
                    actions.add(
                        10,
                        PlayerAction::Sequential(vec![
                            PlayerAction::TakeShelter,
                            PlayerAction::SeekShelter,
                            PlayerAction::Bark(self.motivation(), MotivatorKey::Cold),
                        ]),
                    );
                }

                // The cold just makes you tired for now
                // and maybe sick?
                if self.motivation() > 0.6 {
                    actions.add(5, PlayerAction::BumpMotivator(MotivatorKey::Tiredness));
                    actions.add(2, PlayerAction::BumpMotivator(MotivatorKey::Sickness));
                    actions.add(2, PlayerAction::BumpMotivator(MotivatorKey::Sadness));
                    actions.add(8, PlayerAction::Bark(self.motivation(), MotivatorKey::Cold));
                }

                // and hurt in the absolute worst case
                if self.motivation() > 0.95 {
                    actions.add(5, PlayerAction::BumpMotivator(MotivatorKey::Hurt));
                }
            }
            PlayerFocus::Sleeping { .. } => {
                // Hard to sleep if its cold
                if self.motivation() > 0.7 {
                    actions.add(
                        5,
                        PlayerAction::Sequential(seq![
                            PlayerAction::Bark(self.motivation(), MotivatorKey::Cold),
                            PlayerAction::WakeUp,
                        ]),
                    );
                }
            }
            _ => {}
        }
    }
}

impl Signal for Sadness {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
        match ctx.focus {
            PlayerFocus::Unfocused => {
                if self.motivation() > 0.0 {
                    actions.add(
                        5,
                        PlayerAction::Bark(self.motivation(), MotivatorKey::Sadness),
                    );
                    actions.add(5, PlayerAction::ReduceMotivator(MotivatorKey::Sadness));
                }
            }
            _ => {}
        }
    }
}

// // Here's the idea with friendliness
// // 0% -> actively misanthropic
// // 30% -> will respond if talked to
// // 66% -> will start converstaions
// // 100% -> talks to everything
// impl Signal for Friendliness {
//     fn act_on(&self, ctx: &PlayerActionContext) -> Vec<(usize, PlayerAction)> {
//         let mut actions = Vec::new();

//         // IDEAS:
//         // - talk to some random being at location
//         //   > builds up friendliness relation with that entity
//         // - share some resource with a being at location that sufficiently friendly with
//         // - I kind of want a backup action though if thats not possible, what could that be?

//         match ctx.focus {
//             PlayerFocus::Unfocused => {
//                 if self.motivation()> 0.6 {
//                     actions.push((
//                         10,
//                         PlayerAction::Sequential(seq!(
//                             PlayerAction::TalkWithBeing {
//                                 try_cannot_respond: self.motivation()> 0.9
//                             },
//                             PlayerAction::GoToAdjacent(
//                                 GameLogBody::EntityTrackBeing,
//                                 create_markers!(Being)
//                             )
//                         )),
//                     ));
//                 }

//                 // If not friendly, move away from people
//                 if self.motivation()< 0.33 {
//                     actions.push((
//                         10,
//                         PlayerAction::MoveAwayFrom(
//                             GameLogBody::EntityAvoid,
//                             create_markers!(Player),
//                         ),
//                     ));
//                 }
//             }
//             PlayerFocus::Discussion { interest, .. } => {
//                 // For now just chat
//                 if self.motivation()> 0.6 {
//                     actions.push((10, PlayerAction::Discussion(DiscussionAction::LightChat)));
//                 }

//                 // or chat about something heavier if more interested & friendly
//                 if interest > 5 && self.motivation()> 0.6 {
//                     actions.push((20, PlayerAction::Discussion(DiscussionAction::HeavyChat)));
//                 }

//                 // And if less friendly, also lose interest potentially
//                 if self.motivation()< 0.6 {
//                     actions.push((5, PlayerAction::Discussion(DiscussionAction::LoseInterest)));
//                 }
//             }
//             _ => {}
//         }

//         actions
//     }
// }
