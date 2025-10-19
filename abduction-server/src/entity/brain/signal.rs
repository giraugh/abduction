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

/// Helper for a dynamic signal object. This allows for wrapping and owning where required (`Boxed`) or
/// just passing a reference when possible (`Ref`)
#[derive(Debug)]
pub enum SignalRef<'a> {
    Boxed(Box<dyn Signal>),
    Ref(&'a dyn Signal),
}

impl<'a> SignalRef<'a> {
    pub fn boxed<T: Signal + 'static>(value: T) -> Self {
        Self::Boxed(Box::new(value) as Box<dyn Signal>)
    }

    pub fn reference<T: Signal>(value: &'a T) -> Self {
        Self::Ref(value)
    }
}

impl Signal for SignalRef<'_> {
    fn act_on(&self, ctx: &PlayerActionContext) -> Vec<(usize, PlayerAction)> {
        match self {
            SignalRef::Boxed(signal) => signal.act_on(ctx),
            SignalRef::Ref(signal) => signal.act_on(ctx),
        }
    }
}
