use crate::entity::brain::focus::PlayerFocus;
use crate::entity::{EntityId, EntityMarker};
use crate::hex::AxialHexDirection;
use crate::logs::GameLogBody;

use super::discussion::DiscussionAction;
use super::motivator::MotivatorKey;

#[derive(Clone, Debug)]
pub enum PlayerAction {
    /// No-op
    /// "<player> twiddles their thumbs" etc
    /// (This always causes the "NoEffect" result)
    Nothing,

    /// Increase some motivator by the sensitivity
    BumpMotivator(MotivatorKey),

    /// Decrease some motivator by the sensitivity
    ReduceMotivator(MotivatorKey),

    /// Try each action in the list until one works
    Sequential(Vec<PlayerAction>),

    /// Talk to some being at current location
    /// (if other entity can_respond, will enter a discussion focus)
    TalkWithBeing {
        /// Also talk to e.g animals?
        try_cannot_respond: bool,
    },

    /// When in a discussion focus, do related actions
    Discussion(DiscussionAction),

    /// Travel towards (the nearest?) hex which has an entity with any of the given markers
    /// NOTE: if already at such a location, this will do nothing (and cause NoEffect)
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    GoTowards(GameLogBody, Vec<EntityMarker>),

    /// Move to an adjacent hex where an entity resides with any of the given markers
    /// NOTE: if already at such a location, this will do nothing (and cause NoEffect)
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    GoToAdjacent(GameLogBody, Vec<EntityMarker>),

    /// If there is an entity with one of the given tags at current location, player will move elsewhere
    /// NOTE: requires a log that will be emited interstitially if a suitable hex can be found
    MoveAwayFrom(GameLogBody, Vec<EntityMarker>),

    /// Die and be removed from the game
    Death,

    /// Exclaim about a high motivator of some kind
    Bark(f32, MotivatorKey),

    /// Move to a new hex
    Move(AxialHexDirection),

    /// Attempt to eat any food entity at current location
    ConsumeFood {
        try_dubious: bool,
        try_morally_wrong: bool,
    },

    /// Keep sleeping zzzzz
    /// if not already in a sleep focus, will enter one
    Sleep,

    /// If Sleeping, wake up
    WakeUp,

    /// Drink from a water source at current location
    /// (including water that looks bad?)
    DrinkFromWaterSource { try_dubious: bool },
}

#[derive(Clone, Debug)]
pub enum PlayerActionResult {
    /// Something that happens to the world as a result of player action
    SideEffect(PlayerActionSideEffect),

    /// Action had no effect
    /// (e.g try to eat food but there isnt any)
    NoEffect,

    /// Action succeeded (even if nothing happens)
    Ok,
}

impl PlayerActionResult {
    /// Get the side effect if there is one
    pub fn side_effect(self) -> Option<PlayerActionSideEffect> {
        match self {
            PlayerActionResult::SideEffect(player_action_side_effect) => {
                Some(player_action_side_effect)
            }
            PlayerActionResult::NoEffect => None,
            PlayerActionResult::Ok => None,
        }
    }
}

/// Something that happens to the world as a result of player action
#[derive(Clone, Debug)]
pub enum PlayerActionSideEffect {
    /// The player itself dies
    Death,

    /// Remove some other entity (e.g when eating food)
    RemoveOther(EntityId),

    /// Set some other entities focus
    SetFocus {
        entity_id: EntityId,
        focus: PlayerFocus,
    },
}

impl PlayerAction {
    #[inline(always)]
    pub const fn all_movements() -> &'static [Self] {
        use PlayerAction::*;
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
