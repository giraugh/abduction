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
            sensitivity: rng().random_range(0.2..=1.0),
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
            struct $keys;
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

            $($(
                pub fn $accessor(&mut self) -> &mut MotivatorData {
                    self.0.get_mut(&MotivatorKey::$keys).expect("Expected motivator $keys to be present")
                }
            )?)*
        }
    }
}

declare_motivators!({ Hunger, Thirst });

pub trait MotivatorBehaviour {
    fn get_weighted_actions() -> Vec<(usize, PlayerAction)>;
}

impl MotivatorBehaviour for Hunger {
    fn get_weighted_actions() -> Vec<(usize, PlayerAction)> {
        // TODO:
        todo!()
    }
}
