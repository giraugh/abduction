use std::collections::HashMap;

use itertools::Itertools;
use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    seq::{IndexedRandom, SliceRandom},
};
use serde::{Deserialize, Serialize};

use crate::{
    create_markers,
    entity::{gen::PropGenerator, Entity, EntityAttributes, EntityLocation, EntityMarker},
    hex::AxialHex,
};

/// A list of required/optional prop generators for a location
#[derive(Debug, Clone, Default)]
pub struct LocPropGenerators {
    /// One entity from each will be generated
    pub required: Vec<PropGenerator>,

    /// Each entity may be generated 0 or more times
    pub optional: Vec<PropGenerator>,
}

impl LocPropGenerators {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn with_required(mut self, generator: PropGenerator) -> Self {
        self.required.push(generator);
        self
    }

    pub fn with_optional(mut self, generator: PropGenerator) -> Self {
        self.optional.push(generator);
        self
    }

    pub fn generate_optional_at(&self, location: AxialHex, mut rng: &mut impl rand::Rng) -> Entity {
        let generator = self.optional.choose(&mut rng).unwrap();
        let mut entity = generator.generate(&mut rng);
        entity.attributes.hex = Some(location);
        entity
    }
}

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
    Lake,
    Hill,
    Mountain,
    SmallHut,
}

// Generation controls
impl LocationKind {
    pub fn markers(&self) -> Vec<EntityMarker> {
        match self {
            LocationKind::Plain => vec![],
            LocationKind::Forest => create_markers!(LushLocation),
            LocationKind::Lake => create_markers!(LowLyingLocation, LushLocation),
            LocationKind::Hill => vec![],
            LocationKind::Mountain => vec![],
            LocationKind::SmallHut => vec![],
        }
    }

    pub fn max_of_kind(&self) -> usize {
        match self {
            LocationKind::Plain => 9999,
            LocationKind::Forest => 9999,
            LocationKind::Lake => 5,
            LocationKind::Hill => 100,
            LocationKind::Mountain => 30,
            LocationKind::SmallHut => 2,
        }
    }

    pub fn adjacency_weight_bonus(&self) -> usize {
        match self {
            LocationKind::Plain => 2,
            LocationKind::Forest => 3,
            LocationKind::Lake => 0,
            LocationKind::Hill => 2,
            LocationKind::Mountain => 2,
            LocationKind::SmallHut => 0,
        }
    }

    pub fn temp_hue(&self) -> f32 {
        match self {
            LocationKind::Plain => 129.0,
            LocationKind::Forest => 154.0,
            LocationKind::Lake => 219.0,
            LocationKind::Hill => 53.0,
            LocationKind::Mountain => 33.0,
            LocationKind::SmallHut => 281.0,
        }
    }

    /// Optionally, a location can be associated with prop
    /// generators which can generate props in this location type
    pub fn prop_generators(&self) -> LocPropGenerators {
        use PropGenerator::*;
        match self {
            // Plains are pretty barren
            LocationKind::Plain => LocPropGenerators::none(),

            // Hills have food but not water
            LocationKind::Hill => LocPropGenerators::default().with_optional(NaturalFood),

            // Forests are lush with lots of food and water
            LocationKind::Forest => LocPropGenerators::default()
                .with_optional(PossiblyPoisonousFood)
                .with_optional(NaturalFood)
                .with_optional(QualityNaturalWaterSource)
                .with_optional(DubiousNaturalWaterSource),

            // Lakes always generate a lake water source and also food in the form of fish
            LocationKind::Lake => LocPropGenerators::default()
                .with_required(Lake)
                .with_optional(Fish),

            // Mountiains are pretty barren but can have a mountain lake
            LocationKind::Mountain => {
                LocPropGenerators::default().with_optional(QualityNaturalWaterSource)
            }

            // Small Hut is a WIP
            LocationKind::SmallHut => LocPropGenerators::none(),
        }
    }
}

impl Biome {
    pub fn all_locations(&self) -> Vec<LocationKind> {
        use LocationKind::*;
        match self {
            Biome::Green => vec![Plain, Forest, Lake, Mountain, Hill, SmallHut],
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
            markers: loc_kind.markers(),
            attributes: EntityAttributes {
                hex: Some(*hex),
                display_color_hue: Some(loc_kind.temp_hue()),
                location: Some(EntityLocation {
                    location_kind: loc_kind,
                }),

                ..Default::default()
            },
            ..Default::default()
        });
    });

    loc_entities
}
