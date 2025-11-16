pub mod background;
pub mod brain;
pub mod gen;
pub mod manager;
pub mod snapshot;
pub mod world;

use std::collections::{HashMap, HashSet};

pub use manager::*;

use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::{
    entity::{
        background::EntityBackground,
        brain::{
            characteristic::{Characteristic, CharacteristicStrength},
            focus::ActorFocus,
            meme::MemeTable,
            motivator::MotivatorTable,
        },
        snapshot::EntityView,
        world::EntityWorld,
    },
    hex::AxialHex,
    location::LocationKind,
    mtch::crew::{EntityCollector, EntityPresenter},
};

/// These are sort of tags that can be associated with an entity
///
/// NOTE: When using these, make sure they dont represent something that may also need data on the entity in the future
///       so for example, a corpse isn't a marker because I also need to store the other entity that died
#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[qubit::ts]
pub enum EntityMarker {
    /// Whether this represents a player agent
    /// Maybe remove this later
    Player,

    /// Whether this shows in the world view by default without searching for it
    Inspectable,

    /// A location which is particularly lush
    /// has a lot of plants etc
    /// typically implies it also generates water sources
    LushLocation,

    /// A location which is low-lying on a world scale
    /// typically but not necessarily implies it has available water (e.g a lake)
    LowLyingLocation,

    /// A being that is a human
    Human,

    /// A being that is an alien
    Alien,

    /// A member of "the crew" - part of the production team
    Crew,

    /// A being that can talk
    CanTalk,

    /// Something alive
    Being,

    /// Whether the player escaped on the ship
    /// Maybe remove this later
    Escaped,

    /// This entity represents a fire
    /// which can spread and be put-out
    Fire,

    /// This entity represents somewhere an entity can shelter
    Shelter,
}

pub type EntityId = String; // TODO: use a uuid

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
#[ts(optional_fields)]
pub struct EntityAttributes {
    /// Nested motivators
    pub motivators: MotivatorTable,

    /// The entity first name
    pub first_name: Option<String>,

    /// The entity family name
    pub family_name: Option<String>,

    /// How old the entity is in years
    pub age: Option<usize>,

    /// Which hex the entity is located in if applicable
    pub hex: Option<AxialHex>,

    /// If set, this entity is a corpse of some previous entity
    pub corpse: Option<EntityId>,

    /// If set, this item is entity as a pickupable item
    pub item: Option<EntityItem>,

    /// If set, this entity is a hazard which can deal damage when interacted with
    pub hazard: Option<EntityHazard>,

    /// If set, this entity represents a location with the given location kind
    pub location: Option<EntityLocation>,

    /// If set, this entity is edible
    pub food: Option<EntityFood>,

    /// If set, this entity is an infinite water source
    pub water_source: Option<EntityWaterSource>,

    /// The current details of the world
    pub world: Option<EntityWorld>,

    /// Current focus
    pub focus: Option<ActorFocus>,

    /// Optionally, a set of characteristics
    pub characteristics: Option<HashMap<Characteristic, CharacteristicStrength>>,

    /// A primary hue to use when displaying this entity
    /// The value is a % out of 100 for use in HSL
    /// (e.g for player dots)
    pub display_color_hue: Option<f32>,

    /// Optionally, a background
    pub background: Option<EntityBackground>,

    /// Optionally, a table of memes (like memetic sharable memories / knowledge)
    #[serde(flatten)]
    pub memes: Option<MemeTable>,

    /// If present, this entity is the presenter
    pub presenter: Option<EntityPresenter>,

    /// If present, this entity is the collector
    pub collector: Option<EntityCollector>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
#[ts(optional_fields)]
pub struct EntityRelations {
    /// Poorly named but this is like "opinion" of another entity
    associates: Option<HashMap<EntityId, EntityAssociate>>,

    /// Entities being held
    inventory: Option<HashSet<EntityId>>,
}

impl EntityRelations {
    pub fn inventory_mut(&mut self) -> &mut HashSet<EntityId> {
        self.inventory.get_or_insert_default()
    }

    pub fn inventory(&self) -> impl Iterator<Item = &EntityId> {
        self.inventory.iter().flat_map(|i| i.iter())
    }

    /// Whether we actively like a given entity
    /// (i.e have a positive non-zero bond)
    pub fn bond(&self, entity_id: &EntityId) -> f32 {
        self.associates
            .as_ref()
            .and_then(|a| a.get(entity_id))
            .map(|a| a.bond)
            .unwrap_or_default()
    }

    pub fn associates(&self) -> impl Iterator<Item = (&String, &EntityAssociate)> {
        self.associates
            .as_ref()
            .into_iter()
            .flat_map(|assocs| assocs.iter())
    }

    /// Whether we dislike a given entity
    /// (i.e have a negative bond)
    pub fn dislike(&self, entity_id: &EntityId) -> bool {
        self.bond(entity_id) < 0.0
    }

    /// Whether we actively like a given entity
    /// (i.e have a positive non-zero bond)
    pub fn like(&self, entity_id: &EntityId) -> bool {
        self.bond(entity_id) > 0.0
    }

    /// Create a new associate relation if it doesnt exist, otherwise strengthen it
    /// NOTE: increases by 1% w/ no current cap
    /// TODO: we prob want the ability to limit the influence of this
    pub fn increase_associate_bond(&mut self, entity_id: &EntityId) {
        let associates = self.associates.get_or_insert(Default::default());
        match associates.entry(entity_id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().bond += 0.01;
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(EntityAssociate { bond: 0.01 });
            }
        }
    }

    /// Create a new associate relation if it doesnt exist, otherwise lower it
    /// NOTE: decreases by 1% w/ no current cap
    pub fn decrease_associate_bond(&mut self, entity_id: &EntityId) {
        let associates = self.associates.get_or_insert(Default::default());
        match associates.entry(entity_id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                occupied_entry.get_mut().bond -= 0.01;
            }
            std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                vacant_entry.insert(EntityAssociate { bond: -0.01 });
            }
        }
    }
}

/// Someone you've talked to and know of
/// Bond is between -1 and 1 for the most part
/// at a bond of 1 or higher, the relation may upgrade into ally etc
/// negative values indicate dislike
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[qubit::ts]
pub struct EntityAssociate {
    bond: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
pub struct EntityHazard {
    /// Damage dealt by this hazard, measures in bumps to a hurt motivator
    pub damage: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[qubit::ts]
pub struct EntityLocation {
    pub location_kind: LocationKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[qubit::ts]
pub struct EntityItem {
    /// How much inventory slots (load) this item "takes up"
    /// abstractly represents its size and weight
    /// most things are `1`
    heft: usize,
}

impl Default for EntityItem {
    fn default() -> Self {
        Self { heft: 1 }
    }
}
/// Consumable food
/// TODO: restructure this to just have seperate sustenance and poison fields
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
pub struct EntityFood {
    /// How filling is this food?
    /// (0-1)
    pub sustenance: f32,

    /// Is it poisonous?
    pub poison: f32,

    /// Is it "wrong" to eat this?
    /// i.e a corpse etc
    pub morally_wrong: bool,
}

impl EntityFood {
    pub fn healthy(rng: &mut impl Rng) -> Self {
        Self {
            sustenance: rng.random_range(0.0..1.0),
            poison: 0.0,
            morally_wrong: false,
        }
    }

    pub fn dubious(rng: &mut impl Rng) -> Self {
        Self {
            sustenance: rng.random_range(0.0..0.5),
            poison: if rng.random_bool(0.7) {
                rng.random_range(0.0..1.0)
            } else {
                0.0
            },
            morally_wrong: false,
        }
    }
}

/// An infinite water source
/// All water is just as good at quenching thirst and all water sources are infinite
/// so we just care about whether its tainted by disease/poison etc
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
pub struct EntityWaterSource {
    /// Poison between 0 and 1 -> 1 is worst poison
    pub poison: f32,
}

impl EntityWaterSource {
    pub fn quality() -> Self {
        Self { poison: 0.0 }
    }

    pub fn dubious(rng: &mut impl Rng) -> Self {
        Self {
            poison: rng.random_range(0.0..1.0),
        }
    }
}

/// A full entity including an id
/// SEE ALSO: `EntityPayload`
#[derive(Debug, Clone, Serialize, Default)]
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
    pub relations: EntityRelations,
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
    pub relations: EntityRelations,
}

impl Entity {
    pub fn id() -> EntityId {
        Uuid::now_v7().hyphenated().to_string()
    }

    /// Get (or insert if not present) a mut reference to this entities meme table
    pub fn memes_mut(&mut self) -> &mut MemeTable {
        self.attributes.memes.get_or_insert_default()
    }

    pub fn characteristic(&self, characteristic: Characteristic) -> CharacteristicStrength {
        self.attributes
            .characteristics
            .as_ref()
            .and_then(|c| c.get(&characteristic).cloned())
            .unwrap_or_default()
    }

    /// TODO: Not confident these lifetimes are right...
    pub fn resolve_inventory<'a>(
        &'a self,
        entity_view: &'a EntityView<'a>,
    ) -> impl Iterator<Item = &'a Entity> {
        let ids = self.relations.inventory();
        ids.filter_map(|entity_id| entity_view.by_id(entity_id))
    }

    pub fn max_inventory_load(&self) -> usize {
        // You get load from characteristic
        match self.characteristic(Characteristic::Strength) {
            CharacteristicStrength::Low => 2,
            CharacteristicStrength::Average => 3,
            CharacteristicStrength::High => 5,
        }

        // TODO: and from having a bag etc
    }

    /// Inventory items take up "slots", of which we have an amount derived from our characteristics
    pub fn available_inventory_load(&self, entity_view: &EntityView) -> usize {
        let current_slots = self
            .resolve_inventory(entity_view)
            .filter_map(|e| e.attributes.item.as_ref().map(|i| i.heft))
            .sum::<usize>();
        let max_slots = self.max_inventory_load();
        max_slots - current_slots
    }
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
macro_rules! create_markers {
    ($($markers: ident),*) => {{
        vec![$($crate::entity::EntityMarker::$markers),*]
    }}
}

#[macro_export]
macro_rules! has_markers {
    ($e: expr, $marker: expr) => {{
        use $crate::entity::EntityMarker::*;
        ($e).markers.contains(&($marker))
    }};
    ($e: expr, $marker: expr, $($markers: expr),+) => {{
        use $crate::entity::EntityMarker::*;
        ($e).markers.contains(&$marker) && (has_markers!($e, $($markers),+))
    }};
}
