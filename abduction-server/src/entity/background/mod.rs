use serde::{Deserialize, Serialize};

pub mod career;
pub mod fear;
pub mod hope;

/// Information on an entity's (prob player) background before they were abducted,
/// where they were from, who they were etc
///
/// mostly for flavour and style but also has some mechanical effects (mostly focused on career)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityBackground {
    // Origin
    country_name: String,
    city_name: String,

    // Mechanical
    career: career::Career,
    is_retired: bool,

    // Physical
    eye_colour: String,
    hair_colour: String,

    // Personal stuff
    fear: fear::Fear,
    hope: hope::Hope,
}
