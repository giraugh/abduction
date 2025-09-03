use std::collections::HashMap;

use itertools::Itertools;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    seq::SliceRandom,
};
use serde::{Deserialize, Serialize};

use crate::{
    entity::{Entity, EntityAttributes, EntityLocation, EntityMarker},
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

impl LocationKind {
    pub fn max_of_kind(&self) -> usize {
        match self {
            LocationKind::Plain => 9999,
            LocationKind::Forest => 9999,
            LocationKind::River => 30,
            LocationKind::Hill => 100,
            LocationKind::Mountain => 30,
            LocationKind::SmallHut => 2,
        }
    }

    pub fn adjacency_weight_bonus(&self) -> usize {
        match self {
            LocationKind::Plain => 2,
            LocationKind::Forest => 3,
            LocationKind::River => 2,
            LocationKind::Hill => 2,
            LocationKind::Mountain => 2,
            LocationKind::SmallHut => 0,
        }
    }

    pub fn temp_hue(&self) -> f32 {
        match self {
            LocationKind::Plain => 129.0,
            LocationKind::Forest => 154.0,
            LocationKind::River => 219.0,
            LocationKind::Hill => 53.0,
            LocationKind::Mountain => 33.0,
            LocationKind::SmallHut => 281.0,
        }
    }
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
    let mut rng = rand::rng();
    let mut locs_by_hex = HashMap::<AxialHex, LocationKind>::new();
    let biome_locs = biome.all_locations();
    let mut loc_entities = Vec::new();

    let mut hexs = AxialHex::all_in_bounds(world_radius);
    hexs.shuffle(&mut rng);
    hexs.iter().for_each(|hex| {
        // Initialise weights to count of adjacent
        let mut location_weights: HashMap<_, _> = hex
            .neighbours()
            .iter()
            .filter_map(|n| locs_by_hex.get(n))
            .counts()
            .into_iter()
            .map(|(&location, weight)| (location, weight * location.adjacency_weight_bonus()))
            .collect();

        // And add one for all possible locations
        // unless there's already too many
        let loc_counts = locs_by_hex.values().counts();
        for &location in &biome_locs {
            match loc_counts
                .get(&location)
                .unwrap_or(&0)
                .cmp(&location.max_of_kind())
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
        let loc_kind = {
            if weights.is_empty() || weights.iter().sum::<usize>() == 0 {
                LocationKind::Plain
            } else {
                let dist = WeightedIndex::new(weights).unwrap();
                locations[dist.sample(&mut rng)]
            }
        };

        // Update the map
        locs_by_hex.insert(*hex, loc_kind);

        // Create an entity
        loc_entities.push(Entity {
            entity_id: Entity::id(),
            name: format!("{loc_kind:?}"), // TODO; impl display or have like a set of possible names or soemthing?
            markers: vec![EntityMarker::Viewable],
            relations: vec![],
            attributes: EntityAttributes {
                hex: Some(*hex),
                location: Some(EntityLocation {
                    location_kind: loc_kind,
                }),

                display_color_hue: Some(loc_kind.temp_hue()),

                // TODO: store the location
                ..Default::default()
            },
        });
    });

    loc_entities
}
