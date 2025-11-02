use super::GameEventKind;
use crate::{
    entity::brain::{
        characteristic::{Characteristic, CharacteristicStrength},
        motivator::{MotivatorKey, Sadness},
        player_action::PlayerAction,
        signal::{Signal, SignalContext, WeightedPlayerActions},
    },
    event::{GameEvent, GameEventTarget},
    logs::GameLogBody,
};

impl Signal for GameEvent {
    fn act_on(&self, ctx: &SignalContext, actions: &mut WeightedPlayerActions) {
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
            GameEventKind::LeaveHex { entity_id } => {
                // TODO
            }
            GameEventKind::ArriveInHex { entity_id } => {
                // Ignore this if its us
                if *entity_id == ctx.entity.entity_id {
                    return;
                }

                // If we are friendly, we might choose to great the entity arriving in the hex
                // NOTE: we need to be able to get entities at this point...
                let we_are_friendly = ctx
                    .entity
                    .characteristic(Characteristic::Friendliness)
                    .is_high();
                let dislike = ctx.entity.relations.dislike(entity_id);

                // If we're a friendly person and we dont dislike this person who showed up, consider greeting them
                if we_are_friendly && !dislike {
                    actions.add(
                        20, // too low?
                        PlayerAction::GreetEntity {
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
                        PlayerAction::MournEntity {
                            entity_id: entity_id.clone(),
                        },
                    );
                }

                // Or just be really upset about it?
                // (but in a non personal way)
                if ctx.entity.characteristic(Characteristic::Resolve).is_low() {
                    actions.add(
                        40, // too low?
                        PlayerAction::Sequential(vec![
                            PlayerAction::Log {
                                other: None,
                                body: GameLogBody::EntityUpsetByDeath,
                            },
                            PlayerAction::Bark(1.0, MotivatorKey::Sadness),
                            PlayerAction::BumpMotivator(MotivatorKey::Sadness),
                        ]),
                    );
                }
            }
        }
    }
}
