use rand::seq::IteratorRandom;
use tracing::{info, warn};

use super::GameEventKind;
use crate::{
    entity::brain::{
        actor_action::ActorAction,
        characteristic::Characteristic,
        discussion::{
            DiscussionAction, DiscussionLeadAction, DiscussionRespondAction, InfoTopic, Opinion,
            PersonalTopic,
        },
        focus::{ActorFocus, BOND_REQ_FOR_PERSONAL_BASE},
        meme::Meme,
        motivator::MotivatorKey,
        signal::{Signal, SignalContext, WeightedActorActions},
    },
    event::{GameEvent, GameEventTarget},
    logs::GameLogBody,
};

impl Signal for GameEvent {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedActorActions) {
        // Where did this event *happen*
        let location = match self.target {
            GameEventTarget::Hex(axial_hex) => Some(axial_hex),
            GameEventTarget::HexSurrounds(axial_hex) => Some(axial_hex),
            _ => None,
        };

        // Did the entity notice this event?
        let did_notice = match &self.notice_conditions {
            // If no conditions, always notice
            None => true,

            // Otherwise, if we have a location, can check for noticing
            Some(conditions) => match location {
                Some(loc) => conditions.iter().any(|cond| cond.test(loc, ctx.entity)),
                // For now, if we dont have a location, but we have conditions which expect one, thats a panic
                None => panic!(
                    "Notice condition requires location but the event target does not imply one"
                ),
            },
        };

        // If not noticed, stop resolving
        if !did_notice {
            return;
        }

        // Then handle the specific type
        match &self.kind {
            GameEventKind::LeaveHex {
                entity_id: _entity_id,
            } => {
                // TODO
            }
            GameEventKind::ArriveInHex { entity_id } => {
                // Ignore this if its us
                if *entity_id == ctx.entity.entity_id {
                    return;
                }

                // If we are focused on something, we dont notice
                // (for now)
                if ctx.focus != ActorFocus::Unfocused {
                    return;
                }

                // If we are friendly, we might choose to great the entity arriving in the hex
                let friendliness = ctx.entity.characteristic(Characteristic::Friendliness);
                let dislike = ctx.entity.relations.dislike(entity_id);

                // If we're a friendly person and we dont dislike this person who showed up, consider greeting them
                // NOTE: if we aren't friendly we may still great them, just less likely
                if !friendliness.is_low() && !dislike {
                    actions.add(
                        if friendliness.is_high() { 30 } else { 5 },
                        ActorAction::GreetEntity {
                            entity_id: entity_id.clone(),
                        },
                    );
                }
            }

            GameEventKind::Death { entity_id } => {
                // Have a mini funeral?
                let empathy = ctx.entity.characteristic(Characteristic::Empathy);
                if empathy.is_high() || (ctx.entity.relations.like(entity_id) && !empathy.is_low())
                {
                    actions.add(
                        40, // too low?
                        ActorAction::MournEntity {
                            entity_id: entity_id.clone(),
                        },
                    );
                }

                // Or just be really upset about it?
                // (but in a non personal way)
                if ctx.entity.characteristic(Characteristic::Resolve).is_low() {
                    actions.add(
                        40, // too low?
                        ActorAction::Sequential(vec![
                            ActorAction::Log {
                                other: None,
                                body: GameLogBody::EntityUpsetByDeath,
                            },
                            ActorAction::Bark(1.0, MotivatorKey::Sadness),
                            ActorAction::BumpMotivator(MotivatorKey::Sadness),
                        ]),
                    );
                }
            }

            GameEventKind::LeadDiscussion {
                entity_id: interlocutor_id,
                action,
            } => {
                let mut rng = rand::rng();
                let memes = ctx.entity.attributes.memes.as_ref().unwrap();

                info!("Seeing lead discussion event {self:?}");

                // TODO: chance we just aren't paying attention rn

                // Respond to what they said, based on what they said
                match action {
                    DiscussionLeadAction::AskOpinionOnEntity {
                        entity_id: subject_id,
                    } => {
                        // Resolve an opinion on that entity
                        //  unknown -> Neutral
                        //  like -> Positive
                        //  dislike -> Negative
                        let opinion = match ctx.entity.relations.bond(subject_id).total_cmp(&0.0) {
                            std::cmp::Ordering::Less => Opinion::Negative,
                            std::cmp::Ordering::Equal => Opinion::Neutral,
                            std::cmp::Ordering::Greater => Opinion::Positive,
                        };

                        // Then respond w/ that
                        actions.add(
                            50,
                            DiscussionAction::Respond(DiscussionRespondAction::GiveOpinion {
                                opinion,
                            })
                            .into(),
                        );
                    }

                    DiscussionLeadAction::AskForInfo { topic: info_topic } => {
                        // Attempt to pull a random relevant meme
                        // Will be None if we dont know any relevant info
                        // NOTE: for locations, we could prob make this choose the closest or something
                        let meme = match info_topic {
                            InfoTopic::WaterSourceLocation => memes
                                .water_source_locations()
                                .choose(&mut rng)
                                .map(Meme::WaterSourceAt),
                            InfoTopic::ShelterLocation => memes
                                .shelter_locations()
                                .choose(&mut rng)
                                .map(Meme::ShelterAt),
                        };

                        // Then respond w/ that
                        actions.add(
                            50,
                            DiscussionAction::Respond(DiscussionRespondAction::GiveInfo {
                                topic: *info_topic,
                                meme,
                            })
                            .into(),
                        );
                    }

                    DiscussionLeadAction::AskPersonal {
                        topic: personal_topic,
                    } => {
                        // First off, is this appropriate? Do we balk at this kind of personal question?
                        // this is also based off our personality
                        // whats our threshold?
                        let mut bond_requirement = BOND_REQ_FOR_PERSONAL_BASE;
                        let openness = ctx.entity.characteristic(Characteristic::Openness);
                        if openness.is_high() {
                            bond_requirement -= 0.2;
                        }
                        if openness.is_low() {
                            bond_requirement += 0.2;
                        }

                        let bond = ctx.entity.relations.bond(interlocutor_id);
                        if bond < bond_requirement {
                            actions.add(
                                50,
                                DiscussionAction::Respond(DiscussionRespondAction::Balk).into(),
                            );
                        } else {
                            let bg = ctx.entity.attributes.background.as_ref().unwrap();
                            let answer = match personal_topic {
                                PersonalTopic::Fear => bg.fear.to_string(),
                                PersonalTopic::Hope => bg.hope.to_string(),
                            };

                            actions.add(
                                50,
                                DiscussionAction::Respond(DiscussionRespondAction::GivePersonal {
                                    topic: *personal_topic,
                                    answer,
                                })
                                .into(),
                            );
                        }
                    }
                }
            }

            GameEventKind::RespondDiscussion { entity_id, action } => {
                match action {
                    DiscussionRespondAction::GiveInfo { meme, .. } => {
                        // Store the meme if we recieved one
                        if let Some(meme) = meme {
                            // Okay so we cant actually *DO* anything in this context but we can add possible actions,
                            // so lets add an action that responds with a "thankyou" log and also adds this meme
                            // (Add a very high weight so that we basically always do that thing)
                            // TODO: maybe some people wouldn't be friendly?
                            actions.add(
                                10000,
                                ActorAction::Sequential(vec![
                                    ActorAction::Log {
                                        other: Some(entity_id.clone()),
                                        body: GameLogBody::EntityThank,
                                    },
                                    ActorAction::StoreMeme(meme.clone()),
                                ]),
                            );
                        }
                    }

                    // TODO: FUTURE: update our opinion
                    DiscussionRespondAction::GiveOpinion { opinion } => todo!(),

                    // Not much for us to do in these cases tbh
                    DiscussionRespondAction::Balk
                    | DiscussionRespondAction::GivePersonal { .. } => {}
                }
            }
        }
    }
}
