use crate::{
    entity::brain::signal::{Signal, SignalContext, WeightedPlayerActions},
    event::GameEvent,
};

impl Signal for GameEvent {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {}
}
