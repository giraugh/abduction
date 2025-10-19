use std::collections::{HashMap, HashSet};

use crate::{
    entity::{
        brain::{
            characteristic::Characteristic,
            signal::{Signal, SignalRef},
        },
        Entity, EntityId,
    },
    hex::AxialHex,
    logs::AsEntityId,
};

/// An event happening in the game
#[derive(Debug, Clone)]
pub struct GameEvent {
    /// What is this event
    kind: GameEventKind,

    /// What can respond to this event?
    target: GameEventTarget,

    /// When set, one of these conditions must be met to respond to this event
    notice_conditions: Option<Vec<NoticeCondition>>,
}

impl Signal for GameEvent {
    fn act_on(
        &self,
        ctx: &crate::entity::brain::signal::PlayerActionContext,
    ) -> Vec<(usize, crate::entity::brain::player_action::PlayerAction)> {
        todo!()
    }
}

/// Some condition for noticing an event
#[derive(Debug, Clone)]
pub enum NoticeCondition {
    /// Relies on some characteristic to notice this, at a given max dist
    /// e.g visual acuity, hearing etc
    Sense {
        max_dist: usize,
        characteristic: Characteristic,
    },
}

/// TODO: this is what implements signal? Or is the location usefull to have?
#[derive(Debug, Clone)]
pub enum GameEventKind {
    // TODO
}

// TODO: should this be inverted? And so we just store it where it targets that thing? like we have a collection that maps hexs to events and we
//       just set that? Tho ig I could use this for the interface and then place it into that map accordingly??
//       Have an entity_events and a hex_events map?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEventTarget {
    /// A specific entity
    Entity(EntityId),

    /// A set of entities
    Entities(HashSet<EntityId>),

    /// A hex
    Hex(AxialHex),

    /// A hex and all its neighbours
    HexSurrounds(AxialHex),

    /// Everything
    Global,
}

/// A store for the events raised during a tick
/// raised events are processed in the next tick
///
/// TODO: it might be nice to have one big pool of `GameEvent` that I then
///       reference from each map, to reduce duplication. Maybe even a memory arena?
///
/// TODO: Events can be added mid-tick but should not be processed then.
///       ig if I just resolve all the signals before acting? more memory tho hmm... tho they are just refs
///       I could have a pending events buffer? That gets swapped in when cleared? and thats when I populate the maps?
#[derive(Debug, Clone, Default)]
pub struct EventStore {
    events_by_entity: HashMap<EntityId, Vec<GameEvent>>,
    events_by_hex: HashMap<AxialHex, Vec<GameEvent>>,
    global_events: Vec<GameEvent>,
}

impl EventStore {
    /// Get the events that are relevant for an entity this tick
    pub fn get_event_signals_for_entity(&self, entity: &Entity) -> impl Iterator<Item = SignalRef> {
        // Start with events just for this entity
        let for_entity = self.events_by_entity.get(entity.id()).into_iter().flatten();

        // Get events for this hex that the entity is at (if relevant)
        let for_hex = entity
            .attributes
            .hex
            .and_then(|hex| self.events_by_hex.get(&hex))
            .into_iter()
            .flatten();

        // Grab events for everyone
        let for_all = self.global_events.iter();

        // Then return them all
        itertools::chain!(for_hex, for_all, for_entity).map(SignalRef::reference)
    }

    pub fn add_event(&mut self, event: GameEvent) {
        match &event.target {
            GameEventTarget::Entity(id) => {
                self.events_by_entity
                    .entry(id.clone())
                    .or_default()
                    .push(event);
            }
            GameEventTarget::Entities(ids) => {
                // it gets mad at me if I do this the standard way so uhh bare with me ig...
                let events = std::iter::repeat_with(|| event.clone());
                for (id, event) in std::iter::zip(ids, events) {
                    self.events_by_entity
                        .entry(id.clone())
                        .or_default()
                        .push(event);
                }
            }

            GameEventTarget::Hex(axial_hex) => {
                self.events_by_hex
                    .entry(*axial_hex)
                    .or_default()
                    .push(event);
            }
            GameEventTarget::HexSurrounds(axial_hex) => {
                // NOTE: this can store events in out-of-bound hexs but we just ignore that
                // they'll never get recalled and then theyll be deleted
                // NOTE: it gets mad at me if I do this the standard way so uhh bare with me ig...
                let hexs = axial_hex.neighbours();
                let events = std::iter::repeat_with(|| event.clone());
                for (hex, event) in std::iter::zip(hexs, events) {
                    self.events_by_hex.entry(hex).or_default().push(event);
                }
            }

            GameEventTarget::Global => {
                self.global_events.push(event);
            }
        }
    }

    /// Remove all events
    pub fn clear(&mut self) {
        self.events_by_entity.clear();
        self.events_by_hex.clear();
        self.global_events.clear();
    }
}
