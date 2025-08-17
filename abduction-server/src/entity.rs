use std::{collections::HashMap, ops::Deref};

use anyhow::Context;
use rand::{rng, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{query_file_as, types::Json};
use tokio::sync::broadcast;
use tracing::info;

use crate::{
    mtch::{MatchId, TickEvent},
    Db,
};

/// Convenient enum representation for entity mutations that
/// is sent to clients
///
/// NOTE: see also `EntityMutation` which is an equivalent struct
///       as represented in the db
#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
pub enum EntityManagerMutation {
    /// Upsert an entity
    SetEntity(Entity),

    /// Delete an entity
    RemoveEntity(EntityId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityMutationType {
    #[serde(rename = "S")]
    Set,
    #[serde(rename = "D")]
    Delete,
}

/// Flattened version of `EntityManagerMutation` for use with the DB
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMutation {
    entity_id: String,
    match_id: String,
    mutation_type: EntityMutationType,
    payload: Option<EntityPayload>,
}

impl EntityMutation {
    pub fn from_entity_manager_mutation(
        match_id: MatchId,
        mutation: EntityManagerMutation,
    ) -> Self {
        match mutation {
            EntityManagerMutation::SetEntity(entity) => Self {
                entity_id: entity.entity_id.clone(),
                match_id,
                mutation_type: EntityMutationType::Set,
                payload: Some(entity.into()),
            },
            EntityManagerMutation::RemoveEntity(entity_id) => Self {
                entity_id,
                match_id,
                mutation_type: EntityMutationType::Delete,
                payload: None,
            },
        }
    }
}

/// Loads entities and manages updating them
///
/// # UPDATING ENTITIES
/// In particular, when an entity is mutated somehow,
/// it is immediately updated in this structs state and we store
/// a mutation to be sent to clients/the db at the end of the tick
pub struct EntityManager {
    /// Map from matchId -> Map<>
    match_entities: HashMap<MatchId, HashMap<EntityId, Entity>>,

    /// Waiting mutations for flush
    pending_mutations: Vec<EntityManagerMutation>,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            match_entities: HashMap::default(),
            pending_mutations: Default::default(),
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
            self.match_entities.insert(match_id.clone(), HashMap::new());
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

    /// Update or create a new entity
    pub fn upsert_entity(&mut self, match_id: &MatchId, entity: Entity) -> anyhow::Result<()> {
        let ents = self
            .match_entities
            .get_mut(match_id)
            .context("Getting entities for given match id")?;

        // Upsert that an entity
        ents.insert(entity.entity_id.clone(), entity.clone());

        // Store a mutation for later
        self.pending_mutations
            .push(EntityManagerMutation::SetEntity(entity));

        Ok(())
    }

    pub fn remove_entity(
        &mut self,
        match_id: &MatchId,
        entity_id: &EntityId,
    ) -> anyhow::Result<()> {
        let ents = self
            .match_entities
            .get_mut(match_id)
            .context("Getting entities for given match id")?;

        // Remove that an entity
        ents.remove(entity_id);

        // Store a mutation for later
        self.pending_mutations
            .push(EntityManagerMutation::RemoveEntity(entity_id.clone()));

        Ok(())
    }

    pub async fn flush_changes(
        &mut self,
        tick_tx: &broadcast::Sender<TickEvent>,
        db: &Db,
    ) -> anyhow::Result<()> {
        // If there are no changes, we dont need to do anything
        if self.pending_mutations.is_empty() {
            return Ok(());
        }

        // Otherwise, drain them all
        let pending_mutations: Vec<_> = self.pending_mutations.drain(0..).collect();

        // Send changes to clients
        tick_tx.send(TickEvent::EntityChanges {
            changes: pending_mutations.clone(),
        })?;

        // Add changes to DB
        for mutation in pending_mutations {
            todo!()
            // let mutation = EntityMutation::from_entity_manager_mutation(match_id, mutation);
            // sqlx::query_file!("queries/add_match_mutations.sql")
            //     .fetch(db)
            //     .await;
        }

        // TODO: currently do nothing, but obviously we *should* do things here
        // in particular, write to the DB and send to clients

        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct AggregatedEntities {
    entity_id: EntityId,
    entity: Option<Json<EntityPayload>>,
}

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

pub type CurrMax<T> = (T, T);
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
