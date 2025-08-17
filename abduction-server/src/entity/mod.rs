pub mod manager;
pub use manager::*;

use rand::{rng, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[qubit::ts]
pub enum EntityMarker {
    /// Whether this represents a player agent
    Player,

    /// Whether this can be viewed in the client
    Viewable,

    /// Whether the player escaped on the ship
    Escaped,
}

pub type EntityId = String; // TODO: use a uuid

/// An attribute which "motivates" behaviour for an entity
/// primarily represented by a single 0-1 float
/// entity can react differently to motivators, so they have a
/// sensitity scalar which attenuates incoming "motiviation"
///
/// e.g if sensitivity is 0 for hunger -> that entity does not need to eat
#[derive(Debug, Clone, Serialize, Deserialize)]
#[qubit::ts]
pub struct Motivator {
    /// 0-1 motivation
    motivation: f32,
    /// 0-1 sensitivity
    sensitivity: f32,
}

impl Motivator {
    /// Get a motivator with randomly defined sensitivity
    pub fn random() -> Self {
        Self {
            motivation: 0.0,
            sensitivity: rng().random_range(0.2..=1.0),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
pub struct EntityAttributes {
    // Motivators...
    pub hurt: Option<Motivator>,
    pub hunger: Option<Motivator>,
    pub thirst: Option<Motivator>,

    /// The entity first name
    pub first_name: Option<String>,

    /// The entity family name
    pub family_name: Option<String>,

    /// How old the entity is in years
    pub age: Option<usize>,

    /// Which hex the entity is located in if applicable
    pub hex: Option<(usize, usize)>,

    /// A primary hue to use when displaying this entity
    /// The value is a % out of 100 for use in HSL
    /// (e.g for player dots)
    pub display_color_hue: Option<f32>,
}

impl EntityAttributes {
    /// Generate random motivators
    pub fn random_motivators() -> Self {
        Self {
            hurt: Some(Motivator::random()),
            hunger: Some(Motivator::random()),
            thirst: Some(Motivator::random()),
            ..Default::default()
        }
    }
}

/// A type of entity relation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[qubit::ts]
pub enum RelationKind {
    Friend,
    Lover,
    Child,
    Ally,
    Parent,
    Holding,
}

/// A full entity including an id
/// SEE ALSO: `EntityPayload`
#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
pub struct Entity {
    /// The id of the entity
    pub entity_id: EntityId,

    /// A required name
    pub name: String,

    /// A set of unique "markers"
    pub markers: Vec<EntityMarker>,

    /// Grab bag of attributes
    pub attributes: EntityAttributes,

    /// Relations with other entities
    pub relations: Vec<(RelationKind, EntityId)>,
}

/// An entity as stored in a payload on an entity_mutation row
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct EntityPayload {
    /// A required name
    pub name: String,

    /// A set of unique "markers"
    pub markers: Vec<EntityMarker>,

    /// Grab bag of attributes
    pub attributes: EntityAttributes,

    /// Relations with other entities
    pub relations: Vec<(RelationKind, EntityId)>,
}

impl EntityPayload {
    pub fn convert_to_entity(self, entity_id: EntityId) -> Entity {
        Entity {
            entity_id,
            attributes: self.attributes,
            markers: self.markers,
            name: self.name,
            relations: self.relations,
        }
    }
}

impl From<Entity> for EntityPayload {
    fn from(value: Entity) -> Self {
        Self {
            attributes: value.attributes,
            markers: value.markers,
            name: value.name,
            relations: value.relations,
        }
    }
}

#[macro_export]
macro_rules! has_markers {
    ($e: expr, $marker: expr) => {{
        use EntityMarker::*;
        ($e).markers.contains(&$marker)
    }};
    ($e: expr, $marker: expr, $($markers: expr),+) => {{
        use EntityMarker::*;
        ($e).markers.contains(&$marker) && (has_markers!($e, $($markers),+))
    }};
}
