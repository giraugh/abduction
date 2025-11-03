use serde::{Deserialize, Serialize};

use crate::entity::{
    brain::{
        characteristic::Characteristic, discussion::DiscussionAction, player_action::PlayerAction,
        signal::Signal,
    },
    EntityId,
};

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

impl Signal for PlayerFocus {
    fn act_on(
        &self,
        ctx: &super::signal::SignalContext,
        actions: &mut super::signal::WeightedPlayerActions,
    ) {
        match self {
            PlayerFocus::Unfocused => {}
            PlayerFocus::Sleeping { .. } => {}
            PlayerFocus::Discussion { interest, .. } => {
                let friendliness = ctx.entity.characteristic(Characteristic::Friendliness);

                // For now just chat
                actions.add(10, PlayerAction::Discussion(DiscussionAction::LightChat));

                // or chat about something heavier if more interested & friendly
                if *interest > 5 && friendliness.is_high() {
                    actions.add(20, PlayerAction::Discussion(DiscussionAction::HeavyChat));
                }

                // And if less friendly, also lose interest potentially
                if !friendliness.is_high() {
                    actions.add(5, PlayerAction::Discussion(DiscussionAction::LoseInterest));
                }
            }
        }
    }
}
