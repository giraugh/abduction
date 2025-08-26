use rand::distr::{weighted::WeightedIndex, Distribution};
use tracing::info;

use crate::{
    entity::{
        motivator::{self},
        Entity,
    },
    hex::AxialHexDirection,
    mtch::MatchConfig,
};

#[derive(Clone, Debug)]
pub enum PlayerAction {
    /// No-op
    /// "<player> twiddles their thumbs" etc
    Nothing,

    /// Die and be removed from the game
    Death,

    /// Move to a new hex
    Move(AxialHexDirection),
}

/// Something that happens to the world as a result of player action
#[derive(Clone, Debug)]
pub enum PlayerActionSideEffect {
    Death,
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
        // Get the weighted actions from each motivator
        let action_weights = self.attributes.motivators.get_weighted_actions();
        let (weights, actions): (Vec<_>, Vec<_>) = action_weights.into_iter().unzip();

        // If we have no actions, do nothing
        if actions.is_empty() {
            return PlayerAction::Nothing;
        }

        // TODO: remove impossible actions such as out-of-bounds movement here

        // Create a weighted distribution over the actions
        let dist = WeightedIndex::new(&weights).unwrap();

        // Sample the distribution
        let mut rng = rand::rng();
        actions[dist.sample(&mut rng)].clone()
    }

    pub fn resolve_action(
        &mut self,
        action: PlayerAction,
        config: &MatchConfig,
    ) -> Option<PlayerActionSideEffect> {
        match &action {
            PlayerAction::Nothing => {}
            PlayerAction::Death => {
                return Some(PlayerActionSideEffect::Death);
            }
            PlayerAction::Move(hex_direction) => {
                let hex = self
                    .attributes
                    .hex
                    .as_mut()
                    .expect("Cannot move without hex attribute");
                let new_hex = *hex + (*hex_direction).into();
                if new_hex.within_bounds(config.world_radius as isize) {
                    *hex = new_hex;
                }
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

        None
    }
}
