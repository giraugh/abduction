use serde::{Deserialize, Serialize};

use crate::entity::EntityId;

/// Entities can focus on a certain task or objective. They can also pull other entities into a focus, affecting both of them.
/// When a focus is active, the action-selection logic is unique.
///
/// This is roughly equivalent to the "state" of a player. But note that most state transitions transition via the "unfocused" state,
/// and it will be common for players to spend long stretches of time w/o a focus.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
#[qubit::ts]
#[non_exhaustive]
pub enum PlayerFocus {
    /// No specific focus
    ///  - address immediate needs
    ///  - potentially start a new focus
    ///  - restore "potential" for focus
    Unfocused,

    /// Sleeping, so can't do most normal actions other than sleeping
    /// but could be woken up by stuff etc
    Sleeping { remaining_turns: usize },

    /// Talking with some other entity (not necessarily a player)
    Discussion {
        /// Id of entity talking to
        with: EntityId,

        /// How interested in the conversation we are
        /// at 0, we stop the conversation (not rude per se)
        interest: usize,
    },
}
