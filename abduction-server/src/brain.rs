use rand::Rng;
use serde::Serialize;

use crate::entity::Entity;

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

impl Entity {
    /// Determine the next action to be taken by an entity
    /// Only applicable for players
    pub fn get_next_action(&self) -> PlayerAction {
        use PlayerAction::*;

        // HACK: For now just chooses a random valid action
        let mut possible_actions = Vec::new();

        // And doing nothing
        possible_actions.push(Nothing);

        // Add movements
        possible_actions.extend(vec![
            Move(HexDirection::East),
            Move(HexDirection::NorthEast),
            Move(HexDirection::SouthEast),
            Move(HexDirection::West),
            Move(HexDirection::NorthWest),
            Move(HexDirection::SouthWest),
        ]);

        // Then choose a random one
        let mut rng = rand::rng();
        let action_index = rng.random_range(0..possible_actions.len());
        possible_actions[action_index].clone()
    }

    pub fn resolve_action(&mut self, action: PlayerAction) {
        match action {
            PlayerAction::Nothing => {
                // possible a "boredom" side-effect?
                // TODO:
                // action_log!("actions/do_nothing", subject = entity.id);
            }
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
    }
}
