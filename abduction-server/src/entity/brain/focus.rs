use rand::Rng;
use serde::{Deserialize, Serialize};
use strum::VariantArray;

use crate::{
    entity::{
        brain::{
            actor_action::ActorAction,
            characteristic::Characteristic,
            discussion::{DiscussionAction, DiscussionLeadAction, InfoTopic, PersonalTopic},
            motivator::{self, MotivatorKey},
            signal::Signal,
        },
        EntityId,
    },
    logs::AsEntityId,
};

pub const BOND_ERROR: f32 = 0.1; // 10% for now
pub const BOND_REQ_FOR_PERSONAL_BASE: f32 = 0.4; // TODO: move this, also check its reasonable

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

        /// When true, we are next to speak, emit speaking actions
        /// When false, wait instead for speaking events from an interlocutor
        ///
        /// When we respond to an interlocutor, we set this to true
        /// when we take a speak action, we unset this
        /// (during resolution)
        is_lead: bool,
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

            ActorFocus::Discussion { is_lead, with, .. } => {
                let Some(my_memes) = ctx.entity.attributes.memes.as_ref() else {
                    tracing::error!("Entity {} has no meme table", ctx.entity.id());
                    return;
                };

                // If we are the lead, we take lead actions
                // (but dont greet, we assume thats already happaned at this point)
                if *is_lead {
                    let interlocutor = ctx.entities.by_id(with).unwrap();
                    let mut lead_actions = Vec::new();

                    // We might always ask about entities that *we know about*
                    let entities_to_ask_about: Vec<_> = ctx
                        .entity
                        .relations
                        .associates()
                        .map(|(id, _)| id)
                        .collect();
                    let opinion_weight =
                        (10.0 / entities_to_ask_about.len() as f32).trunc() as usize;
                    for entity_id in entities_to_ask_about {
                        lead_actions.push((
                            opinion_weight,
                            DiscussionLeadAction::AskOpinionOnEntity(entity_id.to_owned()),
                        ));
                    }

                    // We might ask about information we dont know about
                    // We may also ask even if we do know that info, just with less priority
                    // TODO: there is something to be said about re-asking some questions about information... ig we could
                    //       just skip storing `asked` memes for those topics...
                    let know_of_shelter = my_memes.shelter_locations().count() > 0;
                    let know_of_water_source = my_memes.water_source_locations().count() > 0;
                    let shelter_weight = if know_of_shelter { 5 } else { 20 };
                    let water_weight = if know_of_water_source { 5 } else { 20 };
                    lead_actions.push((
                        shelter_weight,
                        DiscussionLeadAction::AskForInfo(InfoTopic::ShelterLocation),
                    ));
                    lead_actions.push((
                        water_weight,
                        DiscussionLeadAction::AskForInfo(InfoTopic::WaterSourceLocation),
                    ));

                    // During the conversation, we attempt to keep track of the others connection w/ us
                    // if we think we are close enough, we can ask personal questions
                    // but this has variance (+-rng) so we might get it wrong
                    // people also just have different tolerances for responding to personal questions
                    // they just dont want to talk about themselves...
                    let mut estimated_bond = interlocutor.relations.bond(ctx.entity.id())
                        + rand::rng().random_range(-BOND_ERROR..=BOND_ERROR);

                    // If we are friendlier, assume they like us more
                    // (and vice versa)
                    let friendliness = ctx.entity.characteristic(Characteristic::Friendliness);
                    if friendliness.is_high() {
                        estimated_bond += BOND_ERROR;
                    }
                    if friendliness.is_low() {
                        estimated_bond -= BOND_ERROR;
                    }

                    // If we think they like us enough, try to talk about more personal topics
                    if estimated_bond > BOND_REQ_FOR_PERSONAL_BASE {
                        for personal_topic in PersonalTopic::VARIANTS {
                            lead_actions
                                .push((20, DiscussionLeadAction::AskPersonal(*personal_topic)));
                        }
                    }

                    // If there was absolutely nothing to talk about, ig we just lose interest
                    // (we know them too well?? I dunno)
                    if lead_actions.is_empty() {
                        actions.add(5, DiscussionAction::LoseInterest.into());
                    }

                    // Consider any lead actions we haven't done before with this entity
                    for (weight, lead_action) in lead_actions {
                        if !my_memes.asked_before(interlocutor.id(), &lead_action) {
                            actions.add(weight, DiscussionAction::Lead(lead_action).into());
                        }
                    }
                }

                // If we aren't leading, typically we do nothing
                // but in the future we may have stuff for us to do, like nodding along etc
                // (「あいずち」みたい)
                if !is_lead {
                    // For now just do nothing
                    // (which is automatic)
                }
            }
        }
    }
}
