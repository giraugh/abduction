use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

use crate::{create_markers, logs::GameLogBody};

use super::brain::PlayerAction;

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
#[qubit::ts]
pub struct MotivatorData {
    /// 0-1 motivation
    motivation: f32,
    /// 0-1 sensitivity
    sensitivity: f32,
}

/// How to initialise a motivator?
pub enum MotivatorInit {
    /// Motivation at 0 (sensitivity still random)
    Zero,

    /// Completely random 0-1 motivation
    #[allow(unused)]
    Random,

    /// Random 0-1 but split into N levels
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
            pub struct $keys;
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

            /// Get the weighted actions from ALL THE MOTIVATORS
            pub fn get_weighted_actions(&self) -> Vec<(usize, PlayerAction)> {
                let mut actions = Vec::new();
                $(
                    {
                      if let Some(behaviour_data) = self.0.get(&$keys::TABLE_KEY) {
                        actions.extend($keys::get_weighted_actions(behaviour_data.motivation).into_iter());
                      } else {
                          warn!("Entity is missing motivator data for {:?}", $keys::TABLE_KEY);
                      }
                    }
                )*
                actions
            }
        }
    }
}

declare_motivators!({
    // Needs
    Hunger: MotivatorInit::Zero,
    Thirst: MotivatorInit::Zero,
    Boredom: MotivatorInit::Zero,
    Hurt: MotivatorInit::Zero,
    Sickness: MotivatorInit::Zero,
    Tiredness: MotivatorInit::Zero,
    Saturation: MotivatorInit::Zero,
    Cold: MotivatorInit::Zero,
    Sadness: MotivatorInit::Zero,

    // Personality traits that lead to action
    // Aggresion: Random,
    // Planning: Random
    // Exploration: Random

    // TODO: Support a bias here so we can bias towards 33%
    Friendliness: MotivatorInit::RandomDiscrete(3)
});

pub trait MotivatorBehaviour {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)>;
}

impl MotivatorBehaviour for Hunger {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        // The generic plan for finding food
        let seek_food_plan: &[PlayerAction] = &[
            PlayerAction::GoToAdjacent(
                GameLogBody::EntityGoToAdjacentLush,
                create_markers!(LushLocation),
            ),
            PlayerAction::Bark(motivation, MotivatorKey::Hunger),
        ];

        // Eat food if we have it, maybe try finding some
        if motivation > 0.3 {
            actions.push((
                if motivation > 0.7 { 30 } else { 10 },
                PlayerAction::Sequential(seq![
                    PlayerAction::ConsumeFood { try_dubious: false, try_morally_wrong: false };
                    ..seek_food_plan,
                ]),
            ));
        }

        // Bit more desperate, eat bad food if thats all there is
        if motivation > 0.6 {
            actions.push((
                if motivation > 0.7 { 30 } else { 10 },
                PlayerAction::Sequential(seq![
                    PlayerAction::ConsumeFood { try_dubious: false, try_morally_wrong: false },
                    PlayerAction::ConsumeFood { try_dubious: true, try_morally_wrong: false };
                    ..seek_food_plan,
                ]),
            ));
        }

        // if extremely hungry, we'll try absolutely desperate things
        if motivation > 0.9 {
            actions.push((
                10,
                PlayerAction::ConsumeFood {
                    try_dubious: true,
                    try_morally_wrong: true,
                },
            ));
            // actions.push((10, PlayerAction::CannibalizeSelf));
        }

        if motivation > 0.9 {
            actions.push((20, PlayerAction::BumpMotivator(MotivatorKey::Hurt)));
        }

        actions
    }
}

impl MotivatorBehaviour for Thirst {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

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
            PlayerAction::Bark(motivation, MotivatorKey::Thirst),
        ];

        // Little bit thirsty, start trying to get water
        if motivation > 0.4 {
            actions.push((
                20,
                PlayerAction::Sequential(seq![
                    PlayerAction::DrinkFromWaterSource { try_dubious: false }; // Only go in for safe water
                    ..seek_water_plan,
                ]),
            ));
        }

        // Urgent Drinking! Drink whatever we have available
        if motivation > 0.7 {
            actions.push((
                30,
                PlayerAction::Sequential(seq![
                    PlayerAction::DrinkFromWaterSource { try_dubious: false },
                    PlayerAction::DrinkFromWaterSource { try_dubious: true };
                    ..seek_water_plan,
                ]),
            ));
        }

        if motivation > 0.9 {
            actions.push((20, PlayerAction::BumpMotivator(MotivatorKey::Hurt)));
        }

        actions
    }
}

impl MotivatorBehaviour for Boredom {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.5 {
            actions.push((2, PlayerAction::Bark(motivation, MotivatorKey::Boredom)));
        }

        // If bored enough, do a random movement
        if motivation > 0.7 {
            actions.extend(
                PlayerAction::all_movements()
                    .iter()
                    .cloned()
                    .map(|action| (25, action)),
            );
        }

        actions
    }
}

impl MotivatorBehaviour for Hurt {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.5 {
            actions.push((5, PlayerAction::Bark(motivation, MotivatorKey::Hurt)));
            actions.push((2, PlayerAction::BumpMotivator(MotivatorKey::Sadness)));
        }

        // If fully "motivated" then die
        if motivation >= 0.99 {
            actions.push((1000, PlayerAction::Death));
        }

        actions
    }
}

impl MotivatorBehaviour for Sickness {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.0 {
            // Its possible for it to randomly get worse or better
            // slightly favouring getting better
            actions.push((8, PlayerAction::ReduceMotivator(MotivatorKey::Sickness)));
            actions.push((5, PlayerAction::BumpMotivator(MotivatorKey::Sickness)));
        }

        if motivation > 0.5 {
            actions.push((10, PlayerAction::Bark(motivation, MotivatorKey::Sickness)));
        }

        if motivation > 0.8 {
            actions.push((10, PlayerAction::Bark(motivation, MotivatorKey::Sickness)));
            actions.push((10, PlayerAction::BumpMotivator(MotivatorKey::Hurt)));
        }

        actions
    }
}

impl MotivatorBehaviour for Tiredness {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.7 {
            actions.push((10, PlayerAction::Bark(motivation, MotivatorKey::Tiredness)));
        }

        if motivation > 0.8 {
            actions.push((20, PlayerAction::Bark(motivation, MotivatorKey::Tiredness)));
            actions.push((if motivation > 0.95 { 50 } else { 10 }, PlayerAction::Sleep));
        }

        actions
    }
}

// Is wet for whatever reason
impl MotivatorBehaviour for Saturation {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.0 {
            // Complain about being wet
            actions.push((15, PlayerAction::Bark(motivation, MotivatorKey::Saturation)));

            // Just slowly become dry
            actions.push((5, PlayerAction::ReduceMotivator(MotivatorKey::Saturation)));
        }

        if motivation > 0.1 {
            // Get more cold
            actions.push((10, PlayerAction::BumpMotivator(MotivatorKey::Cold)));

            // Maybe get sick
            actions.push((
                if motivation > 0.5 { 10 } else { 20 },
                PlayerAction::BumpMotivator(MotivatorKey::Sickness),
            ));
        }

        actions
    }
}

impl MotivatorBehaviour for Cold {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        // TODO: more intelligent plans like finding shelter etc

        // The cold just makes you tired for now
        // and maybe sick?
        if motivation > 0.5 {
            actions.push((5, PlayerAction::BumpMotivator(MotivatorKey::Tiredness)));
            actions.push((2, PlayerAction::BumpMotivator(MotivatorKey::Sickness)));
            actions.push((2, PlayerAction::BumpMotivator(MotivatorKey::Sadness)));
            actions.push((10, PlayerAction::Bark(motivation, MotivatorKey::Cold)));
        }

        // and hurt in the absolute worst case
        if motivation > 0.95 {
            actions.push((5, PlayerAction::BumpMotivator(MotivatorKey::Hurt)));
        }

        actions
    }
}

impl MotivatorBehaviour for Sadness {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.0 {
            actions.push((5, PlayerAction::Bark(motivation, MotivatorKey::Sadness)));
            actions.push((5, PlayerAction::ReduceMotivator(MotivatorKey::Sadness)));
        }

        actions
    }
}

// Here's the idea with friendliness
// 0% -> actively misanthropic
// 30% -> will respond if talked to
// 66% -> will start converstaions
// 100% -> talks to everything
impl MotivatorBehaviour for Friendliness {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        // IDEAS:
        // - talk to some random being at location
        //   > builds up friendliness relation with that entity
        // - share some resource with a being at location that sufficiently friendly with
        // - I kind of want a backup action though if thats not possible, what could that be?

        if motivation > 0.6 {
            actions.push((
                10,
                PlayerAction::Sequential(seq!(
                    PlayerAction::TalkWithBeing {
                        try_non_human: motivation > 0.9
                    },
                    PlayerAction::GoToAdjacent(
                        GameLogBody::EntityTrackBeing,
                        create_markers!(Being)
                    )
                )),
            ));
        }

        // If not friendly, move away from people
        if motivation < 0.33 {
            actions.push((
                10,
                PlayerAction::MoveAwayFrom(GameLogBody::EntityAvoid, create_markers!(Player)),
            ));
        }

        actions
    }
}
