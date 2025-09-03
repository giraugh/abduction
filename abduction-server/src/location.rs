use std::collections::HashMap;

use itertools::Itertools;
use rand::distr::{weighted::WeightedIndex, Distribution};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entity, EntityAttributes},
    hex::AxialHex,
};

/// Various biomes (effectively location sets)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub enum Biome {
    /// Green forest style environment
    Green,
}

/// A kind of location
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub enum LocationKind {
    Plain,
    Forest,
    River,
    Hill,
    Mountain,
    SmallHut,
}

impl Biome {
    pub fn all_locations(&self) -> Vec<LocationKind> {
        use LocationKind::*;
        match self {
            Biome::Green => vec![Plain, Forest, Mountain, Hill, SmallHut],
        }
    }
}

pub fn generate_locations_for_world(world_radius: isize, biome: Biome) -> Vec<Entity> {
    // Generate an environment entity in each hex
    // For each, choose a random biome, weighted towards existing adjacent biomes if applicable
    const MAX_LOCATION_COUNT: usize = 5;
    const WEIGHT_FROM_ADJACENT: usize = 2;

    let mut rng = rand::rng();
    let locs_by_hex = HashMap::<AxialHex, LocationKind>::new();
    let biome_locs = biome.all_locations();
    let mut loc_entities = Vec::new();
    AxialHex::all_in_bounds(world_radius)
        .iter()
        .for_each(|hex| {
            // Initialise weights to count of adjacent
            let mut location_weights: HashMap<_, _> = hex
                .neighbours()
                .iter()
                .filter_map(|n| locs_by_hex.get(n))
                .counts()
                .into_iter()
                .map(|(&location, weight)| (location, weight * WEIGHT_FROM_ADJACENT))
                .collect();

            // And add one for all possible locations
            // unless there's already too many
            let loc_counts = locs_by_hex.values().counts();
            for &location in &biome_locs {
                match loc_counts
                    .get(&location)
                    .unwrap_or(&0)
                    .cmp(&MAX_LOCATION_COUNT)
                {
                    // We dont have many yet, add more weight
                    std::cmp::Ordering::Less => {
                        *location_weights.entry(location).or_insert(0) += 1;
                    }

                    // We have too many of those, prevent it
                    _ => {
                        *location_weights.entry(location).or_insert(0) = 0;
                    }
                }
            }

            // Now sample the weighted distribution
            let (locations, weights): (Vec<_>, Vec<_>) = location_weights.into_iter().unzip();
            let dist = WeightedIndex::new(weights).unwrap();
            let loc_kind = locations[dist.sample(&mut rng)];

            // Create an entity
            loc_entities.push(Entity {
                entity_id: Entity::id(),
                name: format!("{loc_kind:?}"), // TODO; impl display or have like a set of possible names or soemthing?
                markers: vec![],
                relations: vec![],
                attributes: EntityAttributes {
                    hex: Some(*hex),

                    // TODO: store the location
                    ..Default::default()
                },
            });
        });

    loc_entities
}
