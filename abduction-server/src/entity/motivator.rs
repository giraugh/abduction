use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::warn;

use super::brain::PlayerAction;

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
            sensitivity: rng().random_range(0.01..=0.1),
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

    /// Increment a motivator by the sensitivity (with some scaling factor)
    pub fn bump_scaled<K: MotivatorTableKey>(&mut self, scale: f32) {
        let data = self.0.get_mut(&K::TABLE_KEY).unwrap();
        data.motivation = (data.motivation + data.sensitivity * scale).clamp(0.0, 1.0);
    }

    /// Clear out a motivation, setting it back to 0
    pub fn clear<K: MotivatorTableKey>(&mut self) {
        let data = self.0.get_mut(&K::TABLE_KEY).unwrap();
        data.motivation = 0.0;
    }

    /// Decrement a motivator by the specified amount
    pub fn reduce_by<K: MotivatorTableKey>(&mut self, by: f32) {
        let data = self.0.get_mut(&K::TABLE_KEY).unwrap();
        data.motivation = (data.motivation - by).clamp(0.0, 1.0);
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
                      if let Some(behaviour_data) = self.0.get(&$keys::TABLE_KEY) {
                        actions.extend($keys::get_weighted_actions(behaviour_data.motivation).into_iter());
                      } else {
                          warn!("Entity is missing motivator data for {:?}", $keys::TABLE_KEY);
                      }
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

declare_motivators!({ Hunger, Thirst, Boredom, Hurt, Sickness });

pub trait MotivatorBehaviour {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)>;
}

impl MotivatorBehaviour for Hunger {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.3 {
            actions.push((
                if motivation > 0.5 { 3 } else { 1 },
                PlayerAction::Sequential(vec![
                    PlayerAction::ConsumeFood,
                    PlayerAction::Bark(motivation, MotivatorKey::Hunger),
                ]),
            ));
        }

        if motivation > 0.9 {
            actions.push((1, PlayerAction::HungerPangs));
        }

        actions
    }
}

impl MotivatorBehaviour for Thirst {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.5 {
            actions.push((1, PlayerAction::Bark(motivation, MotivatorKey::Thirst)));
        }

        if motivation > 0.8 {
            actions.push((1, PlayerAction::ThirstPangs));
        }

        actions
    }
}

impl MotivatorBehaviour for Boredom {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.5 {
            actions.push((1, PlayerAction::Bark(motivation, MotivatorKey::Boredom)));
        }

        // If bored enough, do a random movement
        if motivation > 0.7 {
            actions.extend(
                PlayerAction::all_movements()
                    .iter()
                    .cloned()
                    .map(|action| (3, action)),
            );
        }

        actions
    }
}

impl MotivatorBehaviour for Hurt {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.5 {
            actions.push((1, PlayerAction::Bark(motivation, MotivatorKey::Hurt)))
        }

        // If fully "motivated" then die
        if motivation >= 0.99 {
            actions.push((100, PlayerAction::Death));
        }

        actions
    }
}

impl MotivatorBehaviour for Sickness {
    fn get_weighted_actions(motivation: f32) -> Vec<(usize, PlayerAction)> {
        let mut actions = Vec::new();

        if motivation > 0.0 {
            actions.push((1, PlayerAction::Bark(motivation, MotivatorKey::Sickness)))
        }

        // TODO: some equivalent to pangs, basically it should cause hurt

        actions
    }
}
