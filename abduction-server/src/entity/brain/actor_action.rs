use crate::entity::brain::focus::ActorFocus;
use crate::entity::{EntityId, EntityMarker};
use crate::hex::{AxialHex, AxialHexDirection};
use crate::logs::GameLogBody;

use super::discussion::DiscussionAction;
use super::motivator::MotivatorKey;

#[derive(Clone, Debug)]
#[allow(unused)]
pub enum ActorAction {
    /// No-op
    /// "<actor> twiddles their thumbs" etc
    /// (This always causes the "NoEffect" result)
    Nothing,

    /// Add some specific entity to the inventory, if there is room
    PickUpEntity(EntityId),

    /// Retrieve some specific entity from the inventory
    RetrieveEntity(EntityId),

    /// Increase some motivator by the sensitivity
    BumpMotivator(MotivatorKey),

    /// Decrease some motivator by the sensitivity
    ReduceMotivator(MotivatorKey),

    /// Try each action in the list until one works
    Sequential(Vec<ActorAction>),

    /// Greet some specific entity
    /// (go up to them and say hello type beat)
    /// the other entity may or may not respond, and if they `can_talk` then this may
    /// start a discussion focus
    GreetEntity { entity_id: EntityId },

    Log {
        other: Option<EntityId>,
        body: GameLogBody,
    },

    /// Mourn the death of another entity
    /// (get sad, have a little vigil etc)
    MournEntity { entity_id: EntityId },

    /// When in a discussion focus, do related actions
    Discussion(DiscussionAction),

    /// Travel towards a given hex
    /// NOTE: if already at the location, this will do nothing (and cause NoEffect)
    GoTowardsHex(AxialHex),

    /// Travel towards (the nearest?) hex which has an entity with any of the given markers
    /// NOTE: if already at such a location, this will do nothing (and cause NoEffect)
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    GoTowards(GameLogBody, Vec<EntityMarker>),

    /// Move to an adjacent hex where an entity resides with any of the given markers
    /// NOTE: if already at such a location, this will do nothing (and cause NoEffect)
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    GoToAdjacent(GameLogBody, Vec<EntityMarker>),

    /// If there is an entity with one of the given tags at current location, the actor will move elsewhere
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    MoveAwayFrom(GameLogBody, Vec<EntityMarker>),

    /// Die and be removed from the game
    Death,

    /// Exclaim about a high motivator of some kind
    Bark(f32, MotivatorKey),

    /// Move to a new hex
    Move(AxialHexDirection),

    /// Eat some specific food entity
    ConsumeFoodEntity(EntityId),

    /// Attempt to eat a food entity at current location
    ConsumeNearbyFood {
        try_dubious: bool,
        try_morally_wrong: bool,
    },

    /// Attempt to retrieve a food item from inventory if we have some
    RetrieveInventoryFood,

    /// Keep sleeping zzzzz
    /// if not already in a sleep focus, will enter one
    Sleep,

    /// If Sleeping, wake up
    WakeUp,

    /// Drink from a water source at current location
    /// (including water that looks bad?)
    DrinkFromWaterSource { try_dubious: bool },

    /// Enter shelter at current location if possible
    TakeShelter,

    /// Leave current shelter
    LeaveShelter,

    /// Head towards shelter if we know where some is
    SeekKnownShelter,

    /// Head towards water if we know where some is
    SeekKnownWaterSource,
}

#[derive(Clone, Debug)]
pub enum ActorActionResult {
    /// Something that happens to the world as a result of an actor action
    SideEffect(ActorActionSideEffect),

    /// Action had no effect
    /// (e.g try to eat food but there isnt any)
    NoEffect,

    /// Action succeeded (even if nothing happens)
    Ok,
}

impl ActorActionResult {
    /// Get the side effect if there is one
    pub fn side_effect(self) -> Option<ActorActionSideEffect> {
        match self {
            ActorActionResult::SideEffect(action_side_effect) => Some(action_side_effect),
            ActorActionResult::NoEffect => None,
            ActorActionResult::Ok => None,
        }
    }
}

/// Something that happens to the world as a result of an action
#[derive(Clone, Debug)]
pub enum ActorActionSideEffect {
    /// The actor itself dies
    Death,

    /// Remove some other entity (e.g when eating food)
    RemoveOther(EntityId),

    /// For some entity, set its location to the provided hex
    UnbanishOther(EntityId, AxialHex),

    /// For some entity, remove its location such that it doesn't exist in the world
    /// e.g when picking up an item, we banish it
    BanishOther(EntityId),

    /// Set some other entities focus
    SetFocus {
        entity_id: EntityId,
        focus: ActorFocus,
    },
}

impl ActorAction {
    #[inline(always)]
    pub const fn all_movements() -> &'static [Self] {
        use ActorAction::*;
        &[
            Move(AxialHexDirection::East),
            Move(AxialHexDirection::NorthEast),
            Move(AxialHexDirection::SouthEast),
            Move(AxialHexDirection::West),
            Move(AxialHexDirection::NorthWest),
            Move(AxialHexDirection::SouthWest),
        ]
    }
}
