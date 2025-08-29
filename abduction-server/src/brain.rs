use rand::distr::{weighted::WeightedIndex, Distribution};
use tokio::sync::broadcast;

use crate::{
    entity::{
        motivator::{self, MotivatorKey},
        Entity,
    },
    hex::AxialHexDirection,
    logs::{GameLog, GameLogBody},
    mtch::MatchConfig,
};

#[derive(Clone, Debug)]
pub enum PlayerAction {
    /// No-op
    /// "<player> twiddles their thumbs" etc
    Nothing,

    /// Die and be removed from the game
    Death,

    /// Exclaim about a high motivator of some kind
    Bark(f32, MotivatorKey),

    /// Move to a new hex
    Move(AxialHexDirection),

    /// Hurt by lack of food
    HungerPangs,

    /// Hurt by lack of water
    ThirstPangs,
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
        config: &MatchConfig,
        log_tx: &broadcast::Sender<GameLog>,
    ) -> Option<PlayerActionSideEffect> {
        match &action {
            PlayerAction::Nothing => {}

            // Literally die
            PlayerAction::Death => {
                log_tx
                    .send(GameLog::entity(self, GameLogBody::EntityDeath))
                    .unwrap();

                return Some(PlayerActionSideEffect::Death);
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
            }

            PlayerAction::HungerPangs => {
                self.attributes.motivators.bump::<motivator::Hurt>();
            }

            PlayerAction::ThirstPangs => {
                self.attributes.motivators.bump::<motivator::Thirst>();
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

                    log_tx
                        .send(GameLog::entity(
                            self,
                            GameLogBody::EntityMovement { by: *hex_direction },
                        ))
                        .unwrap();
                }
            }
        }

        // If the action was do nothing, get bored, otherwise get less bored
        match action {
            // If we do nothing we get bored
            PlayerAction::Nothing => {
                self.attributes.motivators.bump::<motivator::Boredom>();
            }
            // If we just bark, thats like doing nothing...
            PlayerAction::Bark(..) => {}
            // But if we do *something* then reduce our boredom
            _ => {
                self.attributes.motivators.reduce::<motivator::Boredom>();
            }
        }

        None
    }
}
