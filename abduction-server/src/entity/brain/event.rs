use std::collections::HashSet;

use crate::{entity::EntityId, hex::AxialHex};

#[derive(Debug, Clone)]
pub struct Event {
    kind: EventKind,
    target: EventTarget,
}

/// TODO: this is what implements signal? Or is the location usefull to have?
#[derive(Debug, Clone)]
pub enum EventKind {
    // TODO
}

// TODO: should this be inverted? And so we just store it where it targets that thing? like we have a collection that maps hexs to events and we
//       just set that? Tho ig I could use this for the interface and then place it into that map accordingly??
//       Have an entity_events and a hex_events map?
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventTarget {
    /// A specific entity
    Entity,

    /// A set of entities
    Entities(HashSet<EntityId>),

    /// A hex
    Hex(AxialHex),

    /// A hex and all its neighbours
    HexSurrounds(AxialHex),

    /// Everything
    Global,
}
