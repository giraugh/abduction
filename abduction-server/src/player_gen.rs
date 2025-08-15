use rand::prelude::*;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek};
use std::str::FromStr;
use std::{io::SeekFrom, os::unix::fs::MetadataExt, path::PathBuf};

const NAMES_DIR: &str = "../gather-player-data/output";
const FAMILY_NAMES_PATH: &str = "../gather-player-data/output/family_names.txt";

// Player gen constants
const PLAYER_AGE_RANGE: std::ops::Range<usize> = (18..100);

/// Get a random name from a text file, without loading
/// the whole file ideally
/// NOTE: not using tokio here because this should happen as a batch process
///       at odd times, not during game running
pub fn random_line_from_text_file(path: PathBuf) -> anyhow::Result<String> {
    // Need a source of randomness
    let mut rng = rand::rng();

    // Figure out how large the file is
    // then get a random byte offset
    let metadata = fs::metadata(&path)?;
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
            AgeClass::Young => PathBuf::from_str(NAMES_DIR).unwrap().join("young.txt"),
            AgeClass::Mature => PathBuf::from_str(NAMES_DIR).unwrap().join("mature.txt"),
            AgeClass::Old => PathBuf::from_str(NAMES_DIR).unwrap().join("old.txt"),
        }
    }

    /// Get a random first name that is reasonable for this age range
    pub fn get_random_first_name(&self) -> anyhow::Result<String> {
        let path = self.get_names_path();
        random_line_from_text_file(path)
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

/// Generate a player entity
pub fn generate_player() -> anyhow::Result<()> {
    // Generate an age / class
    let mut rng = rand::rng();
    let age = rng.random_range(PLAYER_AGE_RANGE);
    let age_class = AgeClass::from(age);

    // Generate an age appropriate name
    let first_name = age_class.get_random_first_name()?;
    let family_name = random_line_from_text_file(FAMILY_NAMES_PATH.into())?;
    let player_name = format!("{first_name} {family_name}");

    // TODO..

    Ok(())
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_generate_player() {
        generate_player().unwrap();
        panic!();
    }

    #[test]
    fn test_random_line() {
        let names_path = PathBuf::from_str(FAMILY_NAMES_PATH).unwrap();
        let line = random_line_from_text_file(names_path);
        dbg!(&line);
        assert!(line.is_ok());
        assert!(!line.unwrap().is_empty());
    }
}
