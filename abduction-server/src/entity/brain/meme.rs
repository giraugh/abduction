#![allow(unused)]

use std::{collections::HashSet, convert::Infallible, fmt, str::FromStr};

use anyhow::anyhow;
use itertools::Itertools;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use crate::{entity::EntityId, hex::AxialHex};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub enum Danger {
    #[default]
    Safe,
    Dangerous,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, strum::Display)]
#[qubit::ts]
pub enum Meme {
    // == Opinions on entities ==
    /// Do we think a given entity is safe?
    ///  - can apply to basically anything
    ///  - e.g a source of water, should we drink from it?
    #[strum(to_string = "safe:{0}")]
    EntityIsSafe(EntityId),

    /// Do we think a given entity is dangerous?
    ///  - can apply to basically anything
    ///  - e.g a mushroom, is it poisonous?
    #[strum(to_string = "dangerous:{0}")]
    EntityIsDangerous(EntityId),

    // == Locations ==
    /// We are aware of available shelter at this location
    #[strum(to_string = "shelter_at:{0}")]
    ShelterAt(AxialHex),

    /// We are aware of an available water source at this location
    /// (NOTE: only added for safe water sources)
    #[strum(to_string = "water_source_at:{0}")]
    WaterSourceAt(AxialHex),
}

impl FromStr for Meme {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tag, rest) = s.split_once(":").ok_or(anyhow!("Malformed meme. No tag"))?;
        match tag {
            "safe" => Ok(Meme::EntityIsSafe(rest.parse()?)),
            "dangerous" => Ok(Meme::EntityIsDangerous(rest.parse()?)),
            "shelter_at" => Ok(Meme::ShelterAt(rest.parse()?)),
            "water_source_at" => Ok(Meme::WaterSourceAt(rest.parse()?)),
            _ => Err(anyhow!("Failed to parse meme, unkown tag {tag}")),
        }
    }
}

/// A set of memes that an entity is aware of
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[qubit::ts]
pub struct MemeTable {
    #[ts(as = "Vec<String>")]
    #[serde_as(as = "HashSet<DisplayFromStr>")]
    memes: HashSet<Meme>,
}

impl MemeTable {
    /// Choose a random meme in this table that is not present in some other table
    pub fn sample_shareable(&self, other: &Self, rng: &mut impl rand::Rng) -> Option<Meme> {
        let shareable = self.memes.difference(&other.memes).collect_vec();
        shareable.choose(rng).cloned().cloned()
    }

    pub fn remember_is_safe(&mut self, entity_id: &EntityId) {
        self.insert(Meme::EntityIsSafe(entity_id.clone()));
    }

    pub fn remember_is_dangerous(&mut self, entity_id: &EntityId) {
        self.insert(Meme::EntityIsDangerous(entity_id.clone()));
    }

    pub fn insert(&mut self, meme: Meme) {
        self.memes.insert(meme);
    }

    pub fn remove(&mut self, meme: &Meme) {
        self.memes.remove(meme);
    }

    fn is_safe(&self, entity_id: &EntityId) -> bool {
        self.memes.contains(&Meme::EntityIsSafe(entity_id.clone()))
    }

    fn is_dangerous(&self, entity_id: &EntityId) -> bool {
        self.memes
            .contains(&Meme::EntityIsDangerous(entity_id.clone()))
    }

    /// Check whether we have any safe/danger memes for a given entity
    /// (if we have both, returns None)
    pub fn check_danger(&self, entity_id: &EntityId) -> Option<Danger> {
        match (self.is_safe(entity_id), self.is_dangerous(entity_id)) {
            (true, false) => Some(Danger::Safe),
            (false, true) => Some(Danger::Dangerous),

            // If neither, we dont know
            (false, false) => None,

            // If both, they cancel and we say we dont know
            (true, true) => None,
        }
    }

    /// Do we *not* have explicit evidence that this is dangerous?
    /// (i.e it may be dangerous but we dont know)
    pub fn assumably_safe(&self, entity_id: &EntityId) -> bool {
        self.check_danger(entity_id) != Some(Danger::Dangerous)
    }

    pub fn shelter_locations(&self) -> impl Iterator<Item = AxialHex> + use<'_> {
        self.memes.iter().filter_map(|meme| match meme {
            Meme::ShelterAt(hex) => Some(*hex),
            _ => None,
        })
    }

    pub fn water_source_locations(&self) -> impl Iterator<Item = AxialHex> + use<'_> {
        self.memes.iter().filter_map(|meme| match meme {
            Meme::WaterSourceAt(hex) => Some(*hex),
            _ => None,
        })
    }
}

mod meme_string {}
