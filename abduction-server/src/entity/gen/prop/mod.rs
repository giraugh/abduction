//! Prop is my term for an entity which is not a player or location and is intended to be used as part of a players actions
//! e.g food a player can eat, a boar which can attack them etc
mod data;

use data::*;
use rand::seq::IndexedRandom;

use crate::{
    create_markers,
    entity::{Entity, EntityAttributes, EntityFood, EntityItem, EntityWaterSource},
};

/// These are different generators that can create types of props
/// locations can be associated with prop generators to seed the world in this locations
#[derive(Debug, Clone, Copy)]
pub enum PropGenerator {
    /// Food that you might find in nature,
    NaturalFood,

    /// A lake
    Lake,

    /// Fish that can be found in a large water source
    Fish,

    /// A naturally occuring infinite source of water, guaranteed to be high quality
    QualityNaturalWaterSource,

    /// A naturally occuring infinite source of water, potentially causing sickness
    DubiousNaturalWaterSource,

    /// Food found in nature that might be poisonous
    PossiblyPoisonousFood,
    // TODO: fish, wildlife etc (they are different because must be "caught" to become food)
}

pub fn capitalize(s: &str) -> String {
    format!("{}{}", &s[0..1].to_uppercase(), &s[1..])
}

macro_rules! choice {
    ($rng: expr, $($sources: expr),*) => {{
        let opts = vec![$($sources),*]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        opts.choose($rng).unwrap().clone()
    }};
}

impl PropGenerator {
    pub fn name(&self, rng: &mut impl rand::Rng) -> String {
        match self {
            PropGenerator::NaturalFood => {
                format!(
                    "{} {}",
                    choice!(rng, COLOR, SIZE_SHAPE),
                    choice!(rng, NATURAL_FOOD)
                )
            }
            PropGenerator::PossiblyPoisonousFood => {
                format!(
                    "{} {} {}",
                    choice!(rng, DUBIOUS_FOOD_QUALIFIER), // TODO: I want a way to make this optional
                    choice!(rng, COLOR, SIZE_SHAPE),
                    choice!(rng, POSSIBLY_POISONOUS_FOOD)
                )
            }
            PropGenerator::Fish => {
                format!("{} {}", choice!(rng, COLOR, SIZE_SHAPE), choice!(rng, FISH))
            }
            PropGenerator::QualityNaturalWaterSource => {
                format!(
                    "{} {}",
                    choice!(rng, QUALITY_WATER_SOURCE_QUALIFIER),
                    choice!(rng, NATURAL_WATER_SOURCE)
                )
            }
            PropGenerator::DubiousNaturalWaterSource => {
                format!(
                    "{} {}",
                    choice!(rng, DUBIOUS_WATER_SOURCE_QUALIFIER),
                    choice!(rng, NATURAL_WATER_SOURCE)
                )
            }
            PropGenerator::Lake => {
                format!(
                    "{} lake",
                    choice!(rng, QUALITY_WATER_SOURCE_QUALIFIER, COLOR)
                )
            }
        }
    }

    pub fn generate(&self, rng: &mut impl rand::Rng) -> Entity {
        match self {
            PropGenerator::NaturalFood | PropGenerator::PossiblyPoisonousFood => Entity {
                entity_id: Entity::id(),
                name: capitalize(&self.name(rng)),
                attributes: EntityAttributes {
                    item: Some(EntityItem::default()),
                    food: Some(match self {
                        PropGenerator::NaturalFood => EntityFood::healthy(rng),
                        PropGenerator::PossiblyPoisonousFood => EntityFood::dubious(rng),
                        _ => unreachable!(),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },

            PropGenerator::QualityNaturalWaterSource | PropGenerator::DubiousNaturalWaterSource => {
                Entity {
                    entity_id: Entity::id(),
                    name: capitalize(&self.name(rng)),
                    attributes: EntityAttributes {
                        water_source: Some(match self {
                            PropGenerator::QualityNaturalWaterSource => {
                                EntityWaterSource::quality()
                            }
                            PropGenerator::DubiousNaturalWaterSource => {
                                EntityWaterSource::dubious(rng)
                            }
                            _ => unreachable!(),
                        }),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            }

            PropGenerator::Fish => Entity {
                entity_id: Entity::id(),
                name: capitalize(&self.name(rng)),
                attributes: EntityAttributes {
                    // TODO: in future it may be required to catch fish instead
                    item: Some(EntityItem::default()),
                    food: Some(EntityFood::healthy(rng)),
                    ..Default::default()
                },
                markers: create_markers!(Being), // fish are alive
                ..Default::default()
            },

            PropGenerator::Lake => Entity {
                entity_id: Entity::id(),
                name: capitalize(&self.name(rng)),
                attributes: EntityAttributes {
                    water_source: Some(EntityWaterSource::quality()),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
