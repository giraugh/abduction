use std::collections::HashMap;

use crate::{
    entity::{Entity, EntityId},
    hex::AxialHex,
};

/// A view into an entity snapshot
/// with caches for quickly accessing entities in certain hexs or by id
#[derive(Debug, Clone, Default)]
pub struct EntityView<'a> {
    by_hex: HashMap<AxialHex, Vec<&'a Entity>>,
    by_id: HashMap<EntityId, &'a Entity>,
}

impl<'a> EntityView<'a> {
    pub fn by_id(&'a self, id: &EntityId) -> Option<&'a Entity> {
        self.by_id.get(id).copied()
    }

    pub fn all(&'a self) -> impl Iterator<Item = &'a Entity> {
        self.by_id.values().copied()
    }

    pub fn in_hex(&'a self, hex: AxialHex) -> impl Iterator<Item = &'a Entity> {
        self.by_hex
            .get(&hex)
            .into_iter()
            .flat_map(|ents| ents.iter().copied())
    }

    /// Get all the entities that are adjacent to some hex (but not in that hex itself)
    pub fn adjacent_to_hex(&'a self, hex: AxialHex) -> impl Iterator<Item = &'a Entity> {
        hex.neighbours()
            .into_iter()
            .flat_map(|hex| self.in_hex(hex))
    }
}

#[derive(Debug, Clone)]
pub struct EntitySnapshot {
    entities: Vec<Entity>,
}

impl EntitySnapshot {
    pub fn new(entities: Vec<Entity>) -> Self {
        Self { entities }
    }

    pub fn view(&self) -> EntityView {
        // Initialise
        let mut view = EntityView::default();

        // Add all entities
        for entity in &self.entities {
            // add by id
            view.by_id.insert(entity.entity_id.clone(), entity);

            // add by hex
            if let Some(hex) = entity.attributes.hex {
                view.by_hex.entry(hex).or_default().push(entity);
            }
        }

        view
    }
}
