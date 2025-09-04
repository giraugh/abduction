//! Prop is my term for an entity which is not a player or location and is intended to be used as part of a players actions
//! e.g food a player can eat, a boar which can attack them etc
mod data;

use data::*;
use rand::seq::IndexedRandom;

use crate::entity::{Entity, EntityAttributes, EntityFood};

/// These are different generators that can create types of props
/// locations can be associated with prop generators to seed the world in this locations
pub enum PropGenerator {
    /// Food that you might find in nature,
    NaturalFood,

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
                    choice!(rng, DUBIOUS_QUALIFIER), // TODO: I want a way to make this optional
                    choice!(rng, COLOR, SIZE_SHAPE),
                    choice!(rng, POSSIBLY_POISONOUS_FOOD)
                )
            }
        }
    }

    pub fn generate(&self, rng: &mut impl rand::Rng) -> Entity {
        match self {
            PropGenerator::NaturalFood | PropGenerator::PossiblyPoisonousFood => Entity {
                name: capitalize(&self.name(rng)),
                entity_id: Entity::id(),
                attributes: EntityAttributes {
                    food: Some(match self {
                        PropGenerator::NaturalFood => EntityFood::healthy(rng),
                        PropGenerator::PossiblyPoisonousFood => EntityFood::dubious(rng),
                    }),
                    ..Default::default()
                },
                ..Default::default()
            },
        }
    }
}
