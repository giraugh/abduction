use serde::{Deserialize, Serialize};

pub mod career;
pub mod fear;
pub mod hope;

/// Information on an entity's (prob player) background before they were abducted,
/// where they were from, who they were etc
///
/// mostly for flavour and style but also has some mechanical effects (mostly focused on career)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[qubit::ts]
pub struct EntityBackground {
    // Origin
    pub country_name: String,
    pub city_name: String,

    // Mechanical
    pub career: career::Career,
    pub is_retired: bool,

    // Physical
    pub eye_colour: String,
    pub hair_colour: String,

    // Personal stuff
    pub fear: fear::Fear,
    pub hope: hope::Hope,
}
