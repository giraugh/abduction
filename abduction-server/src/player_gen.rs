use anyhow::Context;
use rand::prelude::*;
use std::env;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek};
use std::sync::LazyLock;
use std::{io::SeekFrom, os::unix::fs::MetadataExt, path::PathBuf};

use crate::create_markers;
use crate::entity::motivator::MotivatorTable;
use crate::entity::{Entity, EntityAttributes};
use crate::hex::AxialHex;

static PLAYER_DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| env::var("PLAYER_DATA_PATH").unwrap().into());

static FAMILY_NAMES_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| PLAYER_DATA_DIR.join("family_names.txt"));

// Player gen constants
const PLAYER_AGE_RANGE: std::ops::Range<usize> = 18..100;

/// Generate a player entity
/// (Returns but does not save to DB)
pub fn generate_player() -> anyhow::Result<Entity> {
    // Generate an age / class
    let mut rng = rand::rng();
    let age = rng.random_range(PLAYER_AGE_RANGE);
    let age_class = AgeClass::from(age);

    // Generate an age appropriate name
    // TODO: could add other things like infix letters "* P. * " or suffix titles "Jr" "Sr" etc
    let first_name = age_class.get_random_first_name()?;
    let family_name = random_line_from_text_file(&FAMILY_NAMES_PATH)?;
    let player_name = format!("{first_name} {family_name}");

    // FUTURE: {
    //    TODO: player bond generation
    //    TODO: take all bonds, filter down by compatability with age etc
    // }

    // Initialise empty relations and standard set of markers for players
    let relations = Default::default();
    let markers = create_markers!(Player, DefaultInspectable, Being);

    // Generate some random player attributes
    // (primarily motivators but a few others)
    let mut attributes = EntityAttributes {
        motivators: MotivatorTable::initialise(),
        ..Default::default()
    };

    // Update the age to what we generated earlier
    attributes.age = Some(age);

    // We store the players "family" seperately to their name in case it changes
    // also lets us look up players by family
    // NOTE: all entities have a name, not just players so this is duplicated
    //       it wont update the entity name if these are changed... its all g
    attributes.first_name = Some(first_name);
    attributes.family_name = Some(family_name);

    // Encode the players location
    // which we default to the world origin
    attributes.hex = Some(AxialHex::ZERO);

    // So we can show them on the screen, assign them a colour
    // fairly arbitrary right now I think...
    // but we could align this with alliances etc later
    attributes.display_color_hue = Some(rng.random_range(0.0..360.0));

    // Create the entity
    let player_entity = Entity {
        entity_id: Entity::id(),
        name: player_name,
        markers,
        relations,
        attributes,
    };

    Ok(player_entity)
}

/// Get a random name from a text file, without loading
/// the whole file ideally
/// NOTE: not using tokio here because this should happen as a batch process
///       at odd times, not during game running
pub fn random_line_from_text_file(path: &PathBuf) -> anyhow::Result<String> {
    // Need a source of randomness
    let mut rng = rand::rng();

    // Figure out how large the file is
    // then get a random byte offset
    let metadata = fs::metadata(path).context(format!("Determining size of file {path:?}"))?;
    let size = metadata.size();
    let offset = rng.random_range(0..size);

    // So, Im thinking we just scan forwards until a newline,
    // then read until the next newline
    // feels like the easiest approach I want to say?
    // NOTE: technically makes it impossible to get the first entry but ... oh well
    //       could it also have issues if it choose a byte entry thats late in the file?
    let file = File::open(path)?;
    let mut file = BufReader::new(file);
    file.seek(SeekFrom::Start(offset))?;
    let bytes: Vec<u8> = file
        .bytes()
        .flatten()
        .skip_while(|c| *c != b'\n')
        .skip(1)
        .take_while(|c| *c != b'\n')
        .collect();
    let line = String::from_utf8(bytes)?;
    Ok(line)
}

/// Roughly splits ages into three bands
enum AgeClass {
    /// Less than 30yrs or so
    Young,
    /// Less than 60yrs or so
    Mature,
    /// Older than 60yrs
    Old,
}

impl AgeClass {
    pub fn get_names_path(&self) -> PathBuf {
        match self {
            AgeClass::Young => PLAYER_DATA_DIR.join("young.txt"),
            AgeClass::Mature => PLAYER_DATA_DIR.join("mature.txt"),
            AgeClass::Old => PLAYER_DATA_DIR.join("old.txt"),
        }
    }

    /// Get a random first name that is reasonable for this age range
    pub fn get_random_first_name(&self) -> anyhow::Result<String> {
        let path = self.get_names_path();
        random_line_from_text_file(&path)
    }
}

impl From<usize> for AgeClass {
    fn from(age: usize) -> Self {
        match age {
            x if x < 30 => Self::Young,
            x if x < 60 => Self::Mature,
            _ => Self::Old,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_generate_player() {
        generate_player().unwrap();
        panic!();
    }

    #[test]
    fn test_random_line() {
        let line = random_line_from_text_file(&FAMILY_NAMES_PATH);
        assert!(line.is_ok());
        assert!(!line.unwrap().is_empty());
    }
}
