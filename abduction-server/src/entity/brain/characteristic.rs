use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Serialize_repr, Deserialize_repr,
)]
#[qubit::ts]
#[ts(as = "usize")]
#[repr(usize)]
pub enum CharacteristicStrength {
    Low = 0,
    #[default]
    Average = 1,
    High = 2,
}

impl CharacteristicStrength {
    pub fn is_high(&self) -> bool {
        *self == CharacteristicStrength::High
    }

    #[allow(unused)]
    pub fn is_low(&self) -> bool {
        *self == CharacteristicStrength::Low
    }
}

/// An entity can have a set of these with varying strengths
/// (Roughly like "stats" but also encodes personality)
///
/// If some entity has no characteristic, its presumed to be MEDIUM
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, strum::EnumIter)]
#[serde(rename_all = "snake_case")]
#[qubit::ts]
pub enum Characteristic {
    // == Personality ==
    /// High -> Someone who goes out of their way to interact with others - great at making friends
    /// Medium -> Friendly enough, returns small talk
    /// Low -> Unfriendly, doesnt respond
    Friendliness,

    /// High -> Strongly empathetic, takes on others pain
    /// Low -> Psychopathic uncaring type
    Empathy,

    /// High -> Picks fights for no reason
    /// Low -> Refuses to fight, flees
    Aggression,

    /// High -> Brave and not easily scared/saddened/worried
    /// Low -> Scared at all times
    Resolve,

    /// High -> Will regularly take stock and make plans to improve situation ahead of time
    /// Medium -> Occasionally plans
    /// Low -> Does not plan
    Planning,

    /// (Roughly this is adventurist spirit)
    /// High -> Will wander off looking for things or poke the bear so to speak
    /// Low -> Stays to what they know
    Curiosity,

    // == Physical Ability ==
    /// High -> Easily muscle through obstacles that require physical strength
    /// Low -> Unable to perform some physical feats
    Strength,

    /// High -> Can travel faster, can flee more easily
    Speed,

    /// High -> Get over challenging obstacles, dodge attacks
    /// Low -> Cannot jump/roll/etc
    Acrobatics,

    /// High -> Great vision, see far at good quality
    /// Low -> Low or impaired vision
    Vision,

    /// High -> Great hearing, hear quiet things
    /// Low -> Impaired hearing
    Hearing,
}

impl Characteristic {
    /// Whether a given characteristic is both:
    /// - more likely to be higher if young
    /// - more likely to be lower if old
    pub fn influenced_by_age(&self) -> bool {
        matches!(
            self,
            Characteristic::Strength
                | Characteristic::Speed
                | Characteristic::Acrobatics
                | Characteristic::Vision
                | Characteristic::Hearing
        )
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn char_strength_order() {
        assert!(CharacteristicStrength::Low < CharacteristicStrength::High);
    }
}
