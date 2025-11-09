use serde::{Deserialize, Serialize};

use crate::entity::{
    brain::{
        actor_action::ActorAction,
        characteristic::Characteristic,
        discussion::DiscussionAction,
        motivator::{self, MotivatorKey},
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
pub enum ActorFocus {
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

    /// Taking shelter in some shelter
    /// - helps reduce cold/wind and reduces their impact
    /// - increases boredom
    /// - blocks certain other actions
    Sheltering { shelter_entity_id: EntityId },
}

impl Signal for ActorFocus {
    fn act_on(
        &self,
        ctx: &super::signal::SignalContext,
        actions: &mut super::signal::WeightedActorActions,
    ) {
        match self {
            ActorFocus::Unfocused => {}
            ActorFocus::Sleeping { .. } => {
                actions.add(10, ActorAction::Sleep);
            }
            ActorFocus::Sheltering { .. } => {
                // Get less cold and wet
                actions.add(5, ActorAction::ReduceMotivator(MotivatorKey::Cold));
                actions.add(5, ActorAction::ReduceMotivator(MotivatorKey::Saturation));

                // When to leave?
                // If we ever zero out both motivators, we always leave
                let cold = ctx
                    .entity
                    .attributes
                    .motivators
                    .get_motivation::<motivator::Cold>()
                    .unwrap_or_default();
                let saturation = ctx
                    .entity
                    .attributes
                    .motivators
                    .get_motivation::<motivator::Saturation>()
                    .unwrap_or_default();
                if cold == saturation && cold == 0.0 {
                    actions.add(10, ActorAction::LeaveShelter);
                }
            }
            ActorFocus::Discussion { interest, .. } => {
                let friendliness = ctx.entity.characteristic(Characteristic::Friendliness);

                // For now just chat
                actions.add(10, ActorAction::Discussion(DiscussionAction::LightChat));

                // or chat about something heavier if more interested & friendly
                if *interest > 5 && friendliness.is_high() {
                    actions.add(20, ActorAction::Discussion(DiscussionAction::HeavyChat));
                }

                // And if less friendly, also lose interest potentially
                if !friendliness.is_high() {
                    actions.add(5, ActorAction::Discussion(DiscussionAction::LoseInterest));
                }
            }
        }
    }
}
