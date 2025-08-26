use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::brain::PlayerAction;

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

impl MotivatorData {
    /// Get a motivator with randomly defined sensitivity
    pub fn random() -> Self {
        Self {
            motivation: 0.0,
            sensitivity: rng().random_range(0.05..=0.5),
        }
    }
}

pub trait MotivatorTableKey {
    const TABLE_KEY: MotivatorKey;
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[qubit::ts]
pub struct MotivatorTable(HashMap<MotivatorKey, MotivatorData>);

impl MotivatorTable {
    pub fn insert<K: MotivatorTableKey>(&mut self, data: MotivatorData) {
        self.0.insert(K::TABLE_KEY, data);
    }

    /// Increment a motivator by the sensitivity
    pub fn bump<K: MotivatorTableKey>(&mut self) {
        let data = self.0.get_mut(&K::TABLE_KEY).unwrap();
        data.motivation = (data.motivation + data.sensitivity).clamp(0.0, 1.0);
    }

    /// Decrement a motivator by the standard rate
    /// for now they always go down by 0.2
    pub fn reduce<K: MotivatorTableKey>(&mut self) {
        let data = self.0.get_mut(&K::TABLE_KEY).unwrap();
        data.motivation = (data.motivation - 0.2).clamp(0.0, 1.0);
    }
}

macro_rules! declare_motivators {
    ({ $($keys:ident $(:$accessor: ident)?),* }) => {

    //($($keys:ident),*) => {
        /// Declare the possible motivator keys
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
        #[serde(rename_all = "snake_case")]
        #[qubit::ts]
        pub enum MotivatorKey {
            $($keys,)*
        }

        // Then declare each struct
        $(
            pub struct $keys;
            impl MotivatorTableKey for $keys {
                const TABLE_KEY: MotivatorKey = MotivatorKey::$keys;
            }
        )*

        // And create a method which gets a random state for each motivator
        impl MotivatorTable {
            pub fn random() -> Self {
                let mut table = Self::default();
                $(table.insert::<$keys>(MotivatorData::random());)*
                table
            }

            /// Get the weighted actions from ALL THE MOTIVATORS
            pub fn get_weighted_actions(&self) -> Vec<(usize, PlayerAction)> {
                let mut actions = Vec::new();
                $(
                    {
                      let behaviour_data = self.0.get(&$keys::TABLE_KEY).unwrap(); // TODO: actually handle missing motivators here
                      actions.extend($keys::get_weighted_actions(behaviour_data.motivation).into_iter());
                    }
                )*
                actions
            }

            $($(
                pub fn $accessor(&mut self) -> &mut MotivatorData {
                    self.0.get_mut(&MotivatorKey::$keys).expect("Expected motivator $keys to be present")
                }
            )?)*
        }
    }
}

declare_motivators!({ Hunger, Thirst, Boredom, Hurt });

// TODO:
//  - there's a world here where a motivator actually wants to emit a list of actions for each weight like
//    suppose you're hungry, I might create a set of actions like (EatFood, FindFood) that will eat food if we have it or otherwise find food
//    but I guess that could be encoded as a EatOrFindFood action... yeah...

pub trait MotivatorBehaviour {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)>;
}

impl MotivatorBehaviour for Hunger {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        // If not hungry, dont vote for anything
        // (this should prob be a breakpoint like 0.1 instead ig)
        if motivation == 0.0 {
            return Vec::new();
        }

        todo!()
    }
}

impl MotivatorBehaviour for Thirst {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        // If not hungry, dont vote for anything
        // (this should prob be a breakpoint like 0.1 instead ig)
        if motivation == 0.0 {
            return Vec::new();
        }

        todo!()
    }
}

impl MotivatorBehaviour for Boredom {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        // If bored enough, do a random movement
        if motivation > 0.5 {
            return PlayerAction::all_movements()
                .iter()
                .cloned()
                .map(|action| (1, action))
                .collect();
        }

        Vec::new()
    }
}

impl MotivatorBehaviour for Hurt {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        // If fully "motivated" then die
        if motivation >= 0.99 {
            return vec![(100, PlayerAction::Death)];
        }

        Vec::new()
    }
}
