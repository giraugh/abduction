use serde::Serialize;

use crate::{
    entity::{
        brain::motivator::MotivatorKey,
        world::{TimeOfDay, WeatherKind},
        Entity, EntityId,
    },
    hex::{AxialHex, AxialHexDirection},
};

#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
pub struct GameLog {
    /// Optionally, somewhere this event happened
    pub hex: Option<AxialHex>,

    /// The entities involved
    /// Typically:
    ///   0 -> entity did an action
    ///   1 -> entity acted upon
    pub involved_entities: Vec<EntityId>,

    /// What happened?
    #[serde(flatten)]
    pub body: GameLogBody,
}

impl GameLog {
    pub fn global(body: GameLogBody) -> Self {
        Self {
            hex: None,
            involved_entities: vec![],
            body,
        }
    }

    pub fn entity(entity: &Entity, body: GameLogBody) -> Self {
        Self {
            hex: entity.attributes.hex,
            involved_entities: vec![entity.entity_id.clone()],
            body,
        }
    }

    /// NOTE: uses hex from entity a
    pub fn entity_pair(entity_a: &Entity, entity_b: &Entity, body: GameLogBody) -> Self {
        Self {
            hex: entity_a.attributes.hex,
            involved_entities: vec![entity_a.entity_id.clone(), entity_b.entity_id.clone()],
            body,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum GameLogBody {
    /// An entity moving from one hex to another
    EntityMovement { by: AxialHexDirection },

    /// The time of day changed
    TimeOfDayChange { time_of_day: TimeOfDay },

    /// The weather changed
    WeatherChange { weather: WeatherKind },

    /// An entity death
    EntityDeath,

    /// Primary entity greets a secondary entity
    /// Includes the bond between them (0 -> unknown before this, 0.5 -> have talked a few times, 1 -> friendly etc)
    EntityGreet { bond: f32 },

    /// Primary entity ignores the secondary entity's attempt at discussion/interaction
    EntityIgnore,

    /// Player is following a beings tracks
    EntityTrackBeing,

    /// Primary entity is avoiding the secondary entity (because they are misanthropic)
    EntityAvoid,

    /// Lightning strikes the ground and creates a fire
    LightningStrike,

    /// An entity letting it be known it has a high motivator e.g:
    ///  high boredom -> "John Smith lets out a big yawn"
    ///  high pain -> "John Smith winces in pain"
    ///  high hunger -> "John Smith's stomach growls"
    EntityMotivatorBark {
        motivation: f32,
        motivator: MotivatorKey,
    },

    /// Primary entity was hit by lightning
    EntityHitByLightning,

    /// Entity is warming up a bit in the sun
    EntityWarmBecauseOfTime,

    /// Entity is getting cold because they are exposed at night
    EntityColdBecauseOfTime,

    /// Entity is getting wet because of rain
    EntitySaturatedBecauseOfRain,

    /// Entity heading for low-lying area
    EntityGoDownhill,

    /// Entity heading to adjacent lush looking location
    EntityGoToAdjacentLush,

    /// Entity fell into a water source and got saturated
    EntityFellInWaterSource,

    /// Entity consumed some food or drank some water that turned out to be low quality
    /// and caused sickness
    EntityComplainAboutTaste,

    /// The primary entity drank from the secondary entity
    EntityDrinkFrom,

    /// The primary entity starts sleeping
    EntityStartSleeping,

    /// The primary entity continues to sleep
    EntityKeepSleeping,

    /// The primary entity stops sleeping
    EntityStopSleeping,

    /// The primary entity hesitates before eating the secondary entity
    EntityHesitateBeforeConsume,

    /// The primary entity consumed the secondary entity
    EntityConsume,

    /// Entity A (a hazard) hurts entity B
    HazardHurt,
}
