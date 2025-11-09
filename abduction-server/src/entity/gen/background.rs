use rand::{
    distr::{weighted::WeightedIndex, Distribution},
    seq::IndexedRandom,
};
use strum::VariantArray;

use crate::entity::{
    background::{career::Career, fear::Fear, hope::Hope, EntityBackground},
    gen::random_city_country_pair,
};

impl EntityBackground {
    pub fn random_for_age(rng: &mut impl rand::Rng, age: usize) -> Self {
        let (city, country) = random_city_country_pair().unwrap();
        Self {
            country_name: country,
            city_name: city,

            career: Career::VARIANTS.choose(rng).unwrap().clone(),
            fear: Fear::VARIANTS.choose(rng).unwrap().clone(),
            hope: Hope::VARIANTS.choose(rng).unwrap().clone(),

            // depends on age
            is_retired: rng.random_bool(is_retired_response(age)),

            // sample from distributions
            eye_colour: sample_from_weighted_pairs(rng, EYE_COLOR_WEIGHTS)
                .unwrap()
                .to_string(),
            hair_colour: sample_from_weighted_pairs(rng, HAIR_COLOR_WEIGHTS)
                .unwrap()
                .to_string(),
        }
    }
}

/// The "response function" governing the chance of being retired for a given input age
/// based on a transformed sigmoid function translated by RETIRE_RESPONSE_TRANS and scaled by the RETIRE_RESPONSE_COMP factor
/// see https://www.desmos.com/calculator/51putmitw5
fn is_retired_response(age: usize) -> f64 {
    const RETIRE_RESPONSE_TRANS: f64 = 63.0;
    const RETIRE_RESPONSE_COMP: f64 = 0.16;

    let age = age as f64;
    let numer = f64::exp(RETIRE_RESPONSE_COMP * (age - RETIRE_RESPONSE_TRANS));
    let denom = 1.0 + numer;
    numer / denom
}

pub const EYE_COLOR_WEIGHTS: &[(usize, &str)] = &[
    (40, "brown"),
    (30, "blue"),
    (18, "hazel"),
    (1, "amber"),
    (1, "black"),
    (1, "violet"),
    (1, "red"),
];

pub const HAIR_COLOR_WEIGHTS: &[(usize, &str)] = &[
    (30, "black"),
    (12, "dark brown"),
    (12, "light brown"),
    (12, "blonde"),
    (10, "grey"),
    (3, "red"),
];

fn sample_from_weighted_pairs<T: Clone>(
    rng: &mut impl rand::Rng,
    pairs: &[(usize, T)],
) -> Option<T> {
    // TODO: can I sort of memoize this?
    let (weights, values): (Vec<usize>, Vec<T>) = pairs.iter().cloned().unzip();
    let dist = WeightedIndex::new(weights).ok()?;

    let index = dist.sample(rng);
    Some(values[index].clone())
}
