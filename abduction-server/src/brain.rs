use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    Rng,
};
use serde::Serialize;

use crate::entity::{
    motivator::{self, MotivatorKey},
    Entity,
};

#[derive(Debug, Clone, Serialize)]
pub enum HexDirection {
    East,
    West,
    NorthEast,
    NorthWest,
    SouthEast,
    SouthWest,
}

impl HexDirection {
    pub fn cartesian_offset(&self) -> (isize, isize) {
        match self {
            HexDirection::East => (1, 0),
            HexDirection::West => (-1, 0),
            HexDirection::NorthEast => (1, -1),
            HexDirection::NorthWest => (0, -1),
            HexDirection::SouthEast => (0, 1),
            HexDirection::SouthWest => (-1, -1),
        }
    }
}

#[derive(Clone, Debug)]
pub enum PlayerAction {
    /// No-op
    /// "<player> twiddles their thumbs" etc
    Nothing,

    /// Move to a new hex
    Move(HexDirection),
}

impl PlayerAction {
    #[inline(always)]
    pub const fn all_movements() -> &'static [Self] {
        use PlayerAction::*;
        &[
            Move(HexDirection::East),
            Move(HexDirection::NorthEast),
            Move(HexDirection::SouthEast),
            Move(HexDirection::West),
            Move(HexDirection::NorthWest),
            Move(HexDirection::SouthWest),
        ]
    }
}

impl Entity {
    /// Determine the next action to be taken by an entity
    /// Only applicable for players
    pub fn get_next_action(&self) -> PlayerAction {
        // Get the weighted actions from each motivator
        let action_weights = self.attributes.motivators.get_weighted_actions();
        let (weights, actions): (Vec<_>, Vec<_>) = action_weights.into_iter().unzip();

        // If we have no actions, do nothing
        if actions.is_empty() {
            return PlayerAction::Nothing;
        }

        // Create a weighted distribution over the actions
        let dist = WeightedIndex::new(&weights).unwrap();

        // Sample the distribution
        let mut rng = rand::rng();
        actions[dist.sample(&mut rng)].clone()
    }

    pub fn resolve_action(&mut self, action: PlayerAction) {
        match &action {
            PlayerAction::Nothing => {}
            PlayerAction::Move(hex_direction) => {
                let hex = self
                    .attributes
                    .hex
                    .as_mut()
                    .expect("Cannot move without hex attribute");
                let offset = hex_direction.cartesian_offset();
                hex.0 += offset.0;
                hex.1 += offset.1;
            }
        }

        // If the action was do nothing, get bored, otherwise get less bored
        match action {
            PlayerAction::Nothing => {
                self.attributes.motivators.bump::<motivator::Boredom>();
            }
            _ => {
                self.attributes.motivators.reduce::<motivator::Boredom>();
            }
        }
    }
}
