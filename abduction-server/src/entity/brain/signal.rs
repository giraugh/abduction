use crate::entity::brain::{focus::PlayerFocus, player_action::PlayerAction};

/// Information available when resolving a signal into actions
#[derive(Debug, Clone)]
pub struct PlayerActionContext {
    pub focus: PlayerFocus,
}

/// Something that a player acts on -> can raise weighted actions
pub trait Signal: std::fmt::Debug {
    fn act_on(&self, ctx: &PlayerActionContext) -> Vec<(usize, PlayerAction)>;
}
