use serde::Serialize;
use tokio::sync::broadcast;

use crate::{entity::EntityManager, Db};

pub type TickId = usize;

/// Event occuring during a tick (sent to clients)
#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TickEvent {
    StartOfTick { tick_id: TickId },
    EndOfTick { tick_id: TickId },
}

/// Perform one game tick
/// When a match is on, this is called every second or so to update the state of the world
pub async fn perform_tick(
    tick_tx: &broadcast::Sender<TickEvent>,
    entity_manager: &mut EntityManager,
    db: &Db,
) {
    // curious...
    // TODO
}
