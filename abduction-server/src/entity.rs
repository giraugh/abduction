use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use sqlx::{query_file_as, types::Json};
use tracing::info;

use crate::Db;

pub struct EntityManager {
    entities: HashMap<EntityId, Entity>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::default(),
        }
    }

    /// TODO: maybe use uuid here for match id
    pub async fn load_entities(&mut self, db: &Db, match_id: &str) {
        self.entities.clear();
        query_file_as!(
            AggregatedEntities,
            "queries/reduce_match_entities.sql",
            match_id
        )
        .fetch_all(db)
        .await
        .unwrap()
        .into_iter()
        .for_each(|AggregatedEntities { entity_id, entity }| {
            self.entities
                .insert(entity_id, entity.unwrap().deref().clone());
        });

        info!("Loaded {} entities", self.entities.len());
    }
}

#[derive(sqlx::FromRow)]
struct AggregatedEntities {
    entity_id: EntityId,
    entity: Option<Json<Entity>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityMarker {
    /// Whether this represents a player agent
    Player,

    /// Whether this can be viewed in the client
    Viewable,
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Entity {
    /// A required name
    name: String,

    /// A set of unique "markers"
    /// not enforced 😔
    markers: Vec<EntityMarker>,

    /// Grab bag of attributes
    attributes: EntityAttributes,

    /// Relations with other entities
    relations: Vec<(RelationKind, EntityId)>,
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
    match_id: String,

    /// (Later) a uuid identifying which entity this applies to
    entity_id: EntityId,

    /// The type of mutation
    /// either it sets the current value of an entity or it
    /// removes one
    mutation_type: MutationType,

    /// When this is a "SET" type, this must be Some()
    payload: Option<Entity>,
    // TODO: timestamp type here too
}
