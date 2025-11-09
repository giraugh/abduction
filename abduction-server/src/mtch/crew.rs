//! Main role of the presentor is to introduce the game at the start, then introduce each player as they warp in
//!  then later they also may comment on stuff as it happens like a grizzly death etc
//!
//! They use a customised (more scripted) action resolution mechanism, but they can still do actions
//! like a player would for the most part, if we ever want them to
//!
//! The co-host is setup similarly with custom action resolution but they can also use .resolve_action etc etc
//! their primary role is to wander around and warp out any corpses so they dont pile up and so they can be added to future games
//! they can travel incredibly quickly, so their descriptions should describe them "sprinting at inhuman speed" and stuff like that

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::{
    create_markers,
    entity::{
        brain::{
            actor_action::{ActorAction, ActorActionResult},
            characteristic::{Characteristic, CharacteristicStrength},
            signal::SignalRef,
        },
        Entity, EntityAttributes, EntityId,
    },
    has_markers,
    hex::AxialHex,
    logs::{GameLog, GameLogBody},
    mtch::ActionCtx,
};

pub fn generate_presenter() -> Entity {
    use Characteristic as C;
    use CharacteristicStrength as CS;

    Entity {
        entity_id: Entity::id(),
        name: "Mr Giraffe".into(),
        markers: create_markers!(Being, Inspectable, Alien, Crew, CanTalk),
        attributes: EntityAttributes {
            presenter: Some(EntityPresenter::default()),
            first_name: Some("??".to_owned()),
            family_name: Some("Giraffe".to_owned()),
            age: Some(999_999),
            hex: Some(AxialHex::ZERO),
            characteristics: Some(HashMap::from([
                (C::Strength, CS::Low),
                (C::Acrobatics, CS::Low),
                (C::Hearing, CS::High),
                (C::Planning, CS::High),
                (C::Resolve, CS::High),
                (C::Strength, CS::High),
                (C::Vision, CS::High),
                (C::Friendliness, CS::High),
            ])),
            display_color_hue: Some(130.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn generate_collector() -> Entity {
    use Characteristic as C;
    use CharacteristicStrength as CS;

    Entity {
        entity_id: Entity::id(),
        name: "Alpy the Collector".into(),
        markers: create_markers!(Being, Inspectable, Alien, Crew, CanTalk),
        attributes: EntityAttributes {
            collector: Some(EntityCollector::default()),
            first_name: Some("Alpy".to_owned()),
            family_name: Some("??".to_owned()),
            age: Some(100),
            hex: Some(AxialHex::ZERO),
            characteristics: Some(HashMap::from([
                (C::Strength, CS::High),
                (C::Acrobatics, CS::High),
                (C::Hearing, CS::Low),
                (C::Planning, CS::High),
                (C::Resolve, CS::High),
                (C::Strength, CS::High),
                (C::Vision, CS::High),
                (C::Friendliness, CS::Low),
                (C::Empathy, CS::Low),
            ])),
            display_color_hue: Some(130.0),
            ..Default::default()
        },
        ..Default::default()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[qubit::ts]
pub struct EntityPresenter {
    wait: usize,
}

impl Default for EntityPresenter {
    fn default() -> Self {
        Self { wait: 10 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
pub struct EntityCollector {
    // TODO
}

#[derive(Debug, Clone)]
pub enum PresenterAction {
    Wait,
    StartWaiting(usize),
    IntroducePlayer(EntityId),
}

impl From<PresenterAction> for ActorAction {
    fn from(value: PresenterAction) -> Self {
        ActorAction::Presenter(value)
    }
}

impl Entity {
    pub fn get_next_action_as_presenter<'a>(
        &'a self,
        ctx: &ActionCtx,
        _event_signals: impl Iterator<Item = SignalRef<'a>>,
    ) -> ActorAction {
        // First off, are we truly a presenter? Grab our state
        let Some(presenter @ EntityPresenter { .. }) = &self.attributes.presenter else {
            warn!("Non-presenter tried to act as presenter");
            return ActorAction::Nothing;
        };

        // Waiting? We give some ticks between actions to allow time
        if presenter.wait > 0 {
            return ActorAction::Presenter(PresenterAction::Wait);
        }

        // For now, each action just warp in one player
        // is there a player needing unbanished?
        if let Some(to_warp_entity) = ctx
            .entities
            .all()
            .find(|e| e.attributes.hex.is_none() && has_markers!(e, Player))
        {
            return ActorAction::Sequential(vec![
                ActorAction::ignore(
                    PresenterAction::IntroducePlayer(to_warp_entity.entity_id.clone()).into(),
                ),
                PresenterAction::StartWaiting(10).into(),
                ActorAction::WarpInEntity(to_warp_entity.entity_id.clone()),
            ]);
        }

        ActorAction::Nothing
    }

    pub fn resolve_presenter_action(
        &mut self,
        action: &PresenterAction,
        ctx: &ActionCtx,
    ) -> ActorActionResult {
        match action {
            PresenterAction::Wait => {
                self.attributes.presenter.as_mut().unwrap().wait -= 1;
                ActorActionResult::Ok
            }
            PresenterAction::StartWaiting(ticks) => {
                self.attributes.presenter.as_mut().unwrap().wait = *ticks;
                ActorActionResult::NoEffect // this can be chained to start waiting afterwards
            }
            PresenterAction::IntroducePlayer(entity_id) => {
                let player_entity = ctx.entities.by_id(entity_id).unwrap();
                let name = player_entity.attributes.first_name.as_ref().unwrap();
                let bg = player_entity.attributes.background.as_ref().unwrap();
                let retired = if bg.is_retired { "retired " } else { "" };
                let career = bg.career.to_string();
                let location = bg.location_string();

                ctx.send_log(GameLog::entity(
                    self,
                    GameLogBody::EntitySayExact {
                        quote: format!(
                            "Next up we have {name}. A {retired}{career} warping in from {location}"
                        ),
                    },
                ));

                ActorActionResult::Ok
            }
        }
    }

    pub fn get_next_action_as_collector<'a>(
        &'a self,
        ctx: &ActionCtx,
        _event_signals: impl Iterator<Item = SignalRef<'a>>,
    ) -> ActorAction {
        // First off, are we truly a collector? Grab our state
        let Some(EntityCollector { .. }) = self.attributes.collector else {
            warn!("Non-collector tried to act as collector");
            return ActorAction::Nothing;
        };

        // Find the nearest player corpse if present
        if let Some(corpse_entity) = ctx
            .entities
            .all()
            .filter(|e| e.attributes.corpse.is_some() && e.attributes.hex.is_some())
            .min_by_key(|e| {
                e.attributes
                    .hex
                    .unwrap()
                    .dist_to(self.attributes.hex.unwrap())
            })
        {
            return ActorAction::GoTowardsHex(corpse_entity.attributes.hex.unwrap());
        };

        ActorAction::Nothing
    }
}
