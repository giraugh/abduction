use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use sqlx::{query_file_as, types::Json};
use tracing::info;

use crate::{mtch::MatchId, Db};

pub struct EntityManager {
    /// Map from matchId -> Map<>
    match_entities: HashMap<MatchId, HashMap<EntityId, Entity>>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            match_entities: HashMap::default(),
        }
    }

    pub fn get_entities(&self, match_id: MatchId) -> Option<impl Iterator<Item = &Entity>> {
        self.match_entities.get(&match_id).map(|e| e.values())
    }

    pub async fn load_entities(&mut self, db: &Db, match_id: MatchId) {
        // Either clear or create entities map for the match
        let entity_map = if let Some(ents) = self.match_entities.get_mut(&match_id) {
            ents.clear();
            ents
        } else {
            self.match_entities.insert(match_id, HashMap::new());
            self.match_entities.get_mut(&match_id).unwrap()
        };

        query_file_as!(
            AggregatedEntities,
            "queries/reduce_match_entities.sql",
            match_id,
        )
        .fetch_all(db)
        .await
        .unwrap()
        .into_iter()
        .for_each(|AggregatedEntities { entity_id, entity }| {
            entity_map.insert(
                entity_id.clone(),
                entity.unwrap().deref().clone().convert_to_entity(entity_id),
            );
        });

        info!("Loaded {} entities", entity_map.len());
    }
}

#[derive(sqlx::FromRow)]
struct AggregatedEntities {
    entity_id: EntityId,
    entity: Option<Json<EntityPayload>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityMarker {
    /// Whether this represents a player agent
    Player,

    /// Whether this can be viewed in the client
    Viewable,

    /// Whether the player escaped on the ship
    Escaped,
}

pub type CurrMax<T> = (T, T);
pub type EntityId = String; // TODO: use a uuid

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityAttributes {
    health: Option<CurrMax<usize>>,
    hunger: Option<CurrMax<usize>>,
}

/// A type of entity relation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MutationType {
    #[serde(rename = "S")]
    Set,

    #[serde(rename = "D")]
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMutation {
    /// Unique id for the mutation (auto inc)
    mutation_id: i64,

    /// (Later) a uuid identifying which match the mutation
    /// applies to
    match_id: MatchId,

    /// (Later) a uuid identifying which entity this applies to
    entity_id: EntityId,

    /// The type of mutation
    /// either it sets the current value of an entity or it
    /// removes one
    mutation_type: MutationType,

    /// When this is a "SET" type, this must be Some()
    payload: Option<EntityPayload>,
}
