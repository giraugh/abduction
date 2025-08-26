use std::{
    collections::{HashMap, VecDeque},
    ops::Deref,
};

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use sqlx::{query_file_as, types::Json};
use tokio::sync::broadcast;
use tracing::info;

use super::{Entity, EntityId};
use crate::{
    entity::EntityPayload,
    mtch::{MatchId, TickEvent},
    Db,
};

/// Convenient enum representation for entity mutations that
/// is sent to clients
///
/// NOTE: see also `EntityMutation` which is an equivalent struct
///       as represented in the db
///
/// ALLOW: ignore large size variant as `RemoveEntity` variant is likely much rarer
///        but lets keep an eye on that - could potentially box the SetEntity variant instead
#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
#[allow(clippy::large_enum_variant)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EntityManagerMutation {
    /// Upsert an entity
    SetEntity { entity: Entity },

    /// Delete an entity
    #[allow(unused)]
    RemoveEntity { entity_id: EntityId },
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "text")]
pub enum EntityMutationType {
    #[serde(rename = "S")]
    #[sqlx(rename = "S")]
    Set,
    #[serde(rename = "D")]
    #[sqlx(rename = "D")]
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
        match_id: &MatchId,
        mutation: EntityManagerMutation,
    ) -> Self {
        match mutation {
            EntityManagerMutation::SetEntity { entity } => Self {
                entity_id: entity.entity_id.clone(),
                match_id: match_id.clone(),
                mutation_type: EntityMutationType::Set,
                payload: Some(entity.into()),
            },
            EntityManagerMutation::RemoveEntity { entity_id } => Self {
                entity_id,
                match_id: match_id.clone(),
                mutation_type: EntityMutationType::Delete,
                payload: None,
            },
        }
    }
}

/// Loads entities and manages updating them FOR A GIVEN MATCH
///
/// # UPDATING ENTITIES
/// In particular, when an entity is mutated somehow,
/// it is immediately updated in this structs state and we store
/// a mutation to be sent to clients/the db at the end of the tick
pub struct EntityManager {
    /// The match id
    match_id: MatchId,

    /// Map from entity id to entity
    /// (Note that entity object also has an id)
    entities: HashMap<EntityId, Entity>,

    /// Waiting mutations for flush
    /// (its a queue so we can do optimisations like removing a set for an entity that was also deleted)
    pending_mutations: VecDeque<EntityManagerMutation>,
}

impl EntityManager {
    pub fn new(match_id: &MatchId) -> Self {
        Self {
            match_id: match_id.clone(),
            entities: HashMap::default(),
            pending_mutations: Default::default(),
        }
    }

    pub fn get_all_entities(&self) -> impl Iterator<Item = &Entity> {
        self.entities.values()
    }

    /// Static method which gets entities but does not save them against a manager
    pub async fn load_entities_from_match(
        match_id: &MatchId,
        db: &Db,
    ) -> impl Iterator<Item = Entity> {
        query_file_as!(
            AggregatedEntities,
            "queries/reduce_match_entities.sql",
            match_id,
        )
        .fetch_all(db)
        .await
        .unwrap()
        .into_iter()
        .map(|AggregatedEntities { entity_id, entity }| {
            entity.unwrap().deref().clone().convert_to_entity(entity_id)
        })
    }

    /// Load the entities in from a given match
    pub async fn load_entities(&mut self, db: &Db) {
        let mut loaded = 0;
        query_file_as!(
            AggregatedEntities,
            "queries/reduce_match_entities.sql",
            self.match_id,
        )
        .fetch_all(db)
        .await
        .unwrap()
        .into_iter()
        .for_each(|AggregatedEntities { entity_id, entity }| {
            loaded += 1;
            self.entities.insert(
                entity_id.clone(),
                entity.unwrap().deref().clone().convert_to_entity(entity_id),
            );
        });

        info!("Loaded {} entities", loaded);
    }

    /// Update or create a new entity
    pub fn upsert_entity(&mut self, entity: Entity) -> anyhow::Result<()> {
        // Upsert that an entity
        self.entities
            .insert(entity.entity_id.clone(), entity.clone());

        // Store a mutation for later
        self.pending_mutations
            .push_back(EntityManagerMutation::SetEntity { entity });

        Ok(())
    }

    pub fn mutate<F>(&mut self, entity_id: &EntityId, mutate: F) -> anyhow::Result<()>
    where
        F: FnOnce(&mut Entity),
    {
        // Get that entity
        let entity = self
            .entities
            .get_mut(entity_id)
            .ok_or(anyhow!("No such entity"))?;

        // Update it
        let mut entity_updated = entity.clone();
        mutate(&mut entity_updated);
        self.upsert_entity(entity_updated)?;

        Ok(())
    }

    #[allow(unused)]
    pub fn remove_entity(&mut self, entity_id: &EntityId) -> anyhow::Result<()> {
        // Remove that an entity
        self.entities.remove(entity_id);

        // Store a mutation for later
        self.pending_mutations
            .push_back(EntityManagerMutation::RemoveEntity {
                entity_id: entity_id.clone(),
            });

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
        let mutation_count = pending_mutations.len();

        // TODO: de-dupe mutations affecting the same entity
        //   - If the last op was a `D` -> dont send the initial sets, its just deleted
        //   - If multiple sets for an entity, only keep the last one

        // Send changes to clients
        // TODO: we could do JSON diffs here perhaps...
        tick_tx.send(TickEvent::EntityChanges {
            changes: pending_mutations.clone(),
        })?;

        // Add changes to DB
        for mutation in pending_mutations {
            let mutation = EntityMutation::from_entity_manager_mutation(&self.match_id, mutation);
            let payload = Json(mutation.payload);
            sqlx::query_file!(
                "queries/add_match_mutation.sql",
                mutation.entity_id,
                mutation.match_id,
                mutation.mutation_type,
                payload,
            )
            .execute(db)
            .await
            .context("Failed to persist entity mutation to DB")?;
        }

        info!("Flushed {mutation_count} pending mutation(s)");
        Ok(())
    }
}

#[derive(sqlx::FromRow)]
struct AggregatedEntities {
    entity_id: EntityId,
    entity: Option<Json<EntityPayload>>,
}
