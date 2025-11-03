pub mod builder;
pub mod signal;

use std::collections::{HashMap, HashSet};

use tracing::debug;

use crate::{
    entity::{
        brain::{
            characteristic::{Characteristic, CharacteristicStrength},
            signal::SignalRef,
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

impl NoticeCondition {
    /// Test whether some entity meets the condition given an event location
    pub fn test(&self, location: AxialHex, entity: &Entity) -> bool {
        match self {
            NoticeCondition::Sense {
                max_dist,
                characteristic,
            } => {
                // Need a hex to check dist
                let Some(entity_hex) = entity.attributes.hex else {
                    return false;
                };

                // Check max dist
                let dist = entity_hex.dist_to(location);
                if dist > (*max_dist as isize) {
                    return false;
                }

                // Check characteristic
                entity.characteristic(*characteristic) >= CharacteristicStrength::Average
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum GameEventKind {
    /// Some entity arrives in a new hex
    ArriveInHex { entity_id: EntityId },

    /// Some entity leaves a given hex
    LeaveHex { entity_id: EntityId },

    /// Some entity dies
    Death { entity_id: EntityId },
}

#[allow(unused)]
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
/// FUTURE: it might be nice to have one big pool of `GameEvent` that I then
///       reference from each map, to reduce duplication. Maybe even a memory arena?
///
/// NOTE: Events can be added mid-tick but should not be processed then.
///       ig if I just resolve all the signals before acting? more memory tho hmm... tho they are just refs
///       I could have a pending events buffer? That gets swapped in when cleared? and thats when I populate the maps?
#[derive(Debug, Clone, Default)]
pub struct EventStore {
    /// The store that backs the event references
    events: Vec<GameEvent>,
}

#[derive(Debug, Clone, Default)]
pub struct EventsView<'a> {
    events_by_entity: HashMap<EntityId, Vec<&'a GameEvent>>,
    events_by_hex: HashMap<AxialHex, Vec<&'a GameEvent>>,
    global_events: Vec<&'a GameEvent>,
}

impl<'a> EventsView<'a> {
    fn new(events: &'a Vec<GameEvent>) -> Self {
        let mut view = Self::default();
        for event in events {
            match &event.target {
                GameEventTarget::Entity(id) => {
                    view.events_by_entity
                        .entry(id.clone())
                        .or_default()
                        .push(event);
                }
                GameEventTarget::Entities(ids) => {
                    for id in ids {
                        view.events_by_entity
                            .entry(id.clone())
                            .or_default()
                            .push(event);
                    }
                }

                GameEventTarget::Hex(axial_hex) => {
                    view.events_by_hex
                        .entry(*axial_hex)
                        .or_default()
                        .push(event);
                }
                GameEventTarget::HexSurrounds(axial_hex) => {
                    // NOTE: this can store events in out-of-bound hexs but we just ignore that
                    // they'll never get recalled and then theyll be deleted
                    view.events_by_hex
                        .entry(*axial_hex)
                        .or_default()
                        .push(event);
                    for hex in axial_hex.neighbours() {
                        view.events_by_hex.entry(hex).or_default().push(event);
                    }
                }

                GameEventTarget::Global => {
                    view.global_events.push(event);
                }
            }
        }

        view
    }

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
        itertools::chain!(for_hex, for_all, for_entity).map(|&e| SignalRef::reference(e))
    }
}

impl EventStore {
    /// End the current tick, swapping in the buffer of events as the events for the next tick
    pub fn end_tick(&mut self, pending: Vec<GameEvent>) {
        // Analytics
        debug!("Loading {} events for next tick", pending.len());

        // Currently pending events become the events for the next tick
        let mut pending = pending;
        std::mem::swap(&mut pending, &mut self.events);

        // This will happen anyway but lets do it explicitly
        // at this point, the pending variable is actually the former active events
        drop(pending);
    }

    pub fn view(&self) -> EventsView<'_> {
        EventsView::new(&self.events)
    }
}
