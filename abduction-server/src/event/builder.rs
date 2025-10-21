use std::marker::PhantomData;

use tokio::sync::mpsc::{self};

use super::{GameEvent, GameEventKind, GameEventTarget, NoticeCondition};
use crate::{entity::brain::characteristic::Characteristic, mtch::ActionCtx};

pub struct Yes;
pub struct No;
pub trait _P {}
impl _P for Yes {}
impl _P for No {}

pub struct GameEventBuilder<HasKind: _P, HasTarget: _P> {
    kind: Option<GameEventKind>,
    target: Option<GameEventTarget>,
    notice_conditions: Option<Vec<NoticeCondition>>,
    _k: PhantomData<HasKind>,
    _t: PhantomData<HasTarget>,
}

impl GameEventBuilder<No, No> {
    pub fn new() -> Self {
        Self {
            kind: None,
            target: None,
            notice_conditions: None,
            _k: PhantomData,
            _t: PhantomData,
        }
    }
}

impl<K: _P, T: _P> GameEventBuilder<K, T> {
    pub fn of_kind(self, kind: GameEventKind) -> GameEventBuilder<Yes, T> {
        GameEventBuilder {
            kind: Some(kind),
            target: self.target,
            notice_conditions: self.notice_conditions,
            _k: PhantomData,
            _t: PhantomData,
        }
    }

    pub fn targets(self, target: GameEventTarget) -> GameEventBuilder<K, Yes> {
        GameEventBuilder {
            target: Some(target),
            kind: self.kind,
            notice_conditions: self.notice_conditions,
            _k: PhantomData,
            _t: PhantomData,
        }
    }

    pub fn with_sense(self, characteristic: Characteristic, max_dist: usize) -> Self {
        let mut conds = self.notice_conditions.unwrap_or_default();
        conds.push(NoticeCondition::Sense {
            max_dist,
            characteristic,
        });

        Self {
            notice_conditions: Some(conds),
            ..self
        }
    }
}

impl GameEventBuilder<Yes, Yes> {
    pub fn build(self) -> GameEvent {
        GameEvent {
            kind: self.kind.unwrap(),
            target: self.target.unwrap(),
            notice_conditions: self.notice_conditions,
        }
    }

    pub fn add(self, ctx: &mut ActionCtx) {
        ctx.add_event(self.build());
    }
}
