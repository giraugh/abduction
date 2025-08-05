/// Matches module (called `mtch` for rust reasons)
///
/// The #plan for matches and match sequencing
///
/// MATCHES
/// - Match configurations are stored in the DB
/// - Matches can have a "predecessor" from which the population is pulled from
/// - Match populations (players) are prepared/created before the match actually runs
///
/// SEEDING NEW MATCHES
/// - Over the weekend, "setup" a new match that has no predecessor and a large player count
/// - This match will then have lots of players generated for it
/// - The match will then be scheduled but not run until the Monday.
/// - Add queries and UI such that players can see the next upcoming match.
///
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tokio::sync::broadcast;

use crate::{
    entity::{EntityManager, EntityMarker},
    Db,
};

/// Id for a given match
pub type MatchId = i64;

#[derive(Debug, Clone)]
pub struct OptionalMatchId(MatchId);

/// An id identifying a specific tick
/// NOTE: Not scoped to a match but global for the server
/// NOTE: Tick ids are not unique and may overflow, just helps with debugging and testing
pub type TickId = usize;

/// Event occuring during a tick
/// Is sent to clients so they can display the game in real-time
#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TickEvent {
    StartOfTick { tick_id: TickId },
    EndOfTick { tick_id: TickId },
}

/// The configuration for a given match
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct MatchConfig {
    /// Unique v7 uuid for this match
    match_id: MatchId,

    /// The number of players in the match
    ///  - Players will be copied across from predecessor match if appropriate,
    ///  - otherwise new players will be generated when the match is setup
    player_count: i64,

    /// Optionally, the id of the match preceding
    /// this one. If set, players and some entities may be copied across
    /// #[sqlx(try_from = "Option<String>")]
    preceding_match_id: Option<MatchId>,

    /// When the configuration was created
    created_at: NaiveDateTime,
}

impl MatchConfig {
    /// Get one match config from the db
    pub async fn get(db: &Db, match_id: MatchId) -> sqlx::Result<Self> {
        sqlx::query_file_as!(Self, "queries/get_match_config.sql", match_id)
            .fetch_one(db)
            .await
    }
}

macro_rules! has_markers {
    ($e: expr, $marker: expr) => {{
        use EntityMarker::*;
        ($e).markers.contains(&$marker)
    }};
    ($e: expr, $marker: expr, $($markers: expr),+) => {{
        use EntityMarker::*;
        ($e).markers.contains(&$marker) && (has_markers!($e, $($markers),+))
    }};
}

/// Create a new match
///  set new match id, spawn players etc
///  then save the configuration to the db
///
/// NOTE: This can be done before the match is ready to be run
///       i.e right after the previous match if appropriate.
pub async fn create_new_match(
    match_config: &MatchConfig,
    entity_manager: &mut EntityManager,
    db: &Db,
) {
    // TODO: Add all the unescaped players from the last game
    //       In practice, this just means cloning the entity into the new match
    if let Some(preceding_match_id) = match_config.preceding_match_id {
        entity_manager
            .get_entities(preceding_match_id)
            .expect("No such preceding match {preceding_match_id}")
            .filter(|e| has_markers!(e, Player) && !has_markers!(e, Escaped))
            .for_each(|entity| {
                // TODO
            })
    }

    // TODO: If we dont have enough players for the match configuration,
    //       then generate and add more
}

/// Perform one game tick
/// When a match is on, this is called every second or so to update the state of the world
pub async fn perform_match_tick(
    tick_tx: &broadcast::Sender<TickEvent>,
    entity_manager: &mut EntityManager,
    db: &Db,
) {
    // curious...
    // TODO
}
