use anyhow::Context;
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
use chrono::{Local, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tokio::sync::broadcast;
use tracing::info;
use uuid::Uuid;

use crate::{
    entity::{EntityManager, EntityManagerMutation, EntityMarker},
    has_markers,
    player_gen::generate_player,
    Db,
};

/// Id for a given match
/// (generated as a UUID but its just TEXT, can be anything...)
pub type MatchId = String;

/// An id identifying a specific tick
/// NOTE: Not scoped to a match but global for the server
/// NOTE: Tick ids are not unique and may overflow, just helps with debugging and testing
pub type TickId = usize;

pub struct MatchManager {
    /// There may not always be a match on, but there will always be a match manager
    /// thus this is optional
    current_match: Option<MatchConfig>,
}

impl MatchManager {
    pub fn new() -> Self {
        Self {
            current_match: None,
        }
    }

    /// Update currently loaded match,
    /// load all the entities from it,
    /// etc
    ///
    /// NOTE: use this when resuming a match
    pub async fn load_match(
        &mut self,
        match_config: MatchConfig,
        entity_manager: &mut EntityManager,
        db: &Db,
    ) {
        // Update state for current match
        self.current_match = Some(match_config.clone());

        // And load them now
        entity_manager
            .load_entities(db, match_config.match_id)
            .await;
    }

    /// Load in a match configuration, generating any resources needed for the game
    /// Updates "current match" to this new match config.
    ///
    /// This should only be done once per match, realistically - so prob do it when
    /// the config is created
    ///
    /// NOTE: This can be done before the match is ready to be run
    ///       i.e right after the previous match if appropriate.
    pub async fn initialise_new_match(
        &mut self,
        match_config: MatchConfig,
        entity_manager: &mut EntityManager,
        db: &Db,
    ) -> anyhow::Result<()> {
        // First load that match
        self.load_match(match_config.clone(), entity_manager, db)
            .await;

        // Now we initialise it...
        info!("Initialising match {}", &match_config.match_id);

        // Add all the unescaped players from the last game
        // In practice, this just means cloning the entity into the new match
        let mut existing_players = 0;
        if let Some(preceding_match_id) = &match_config.preceding_match_id {
            entity_manager
                .get_entities(preceding_match_id.clone())
                .expect("No such preceding match {preceding_match_id}")
                .filter(|e| has_markers!(e, Player) && !has_markers!(e, Escaped))
                .for_each(|entity| {
                    existing_players += 1;
                    unimplemented!(); // TODO: actually copy across the entity
                })
        }

        // If we dont have enough players for the match configuration,
        // then generate and add more
        let player_count_to_gen = match_config.player_count - existing_players;
        for _ in 0..player_count_to_gen {
            let player_entity = generate_player()?;
            entity_manager.upsert_entity(&match_config.match_id, player_entity)?;
        }

        // Put players in the desired locations
        // TODO

        Ok(())
    }

    /// Perform one game tick
    /// When a match is on, this is called every second or so to update the state of the world
    pub async fn perform_match_tick(
        &mut self,
        tick_tx: &broadcast::Sender<TickEvent>,
        entity_manager: &mut EntityManager,
        db: &Db,
    ) {
        let Some(match_config) = &self.current_match else {
            panic!("Cannot run match tick without current match");
        };

        // TODO: implement actual actions, agents, world changes etc

        // Flush changes to entities to the DB and to clients
        entity_manager.flush_changes(tick_tx, &db).await.unwrap();
    }
}

/// Event occuring during a tick
/// Is sent to clients so they can display the game in real-time
///
/// TIMING:
///
///  - StartOfTick
///  - (Processing happens on server)
///  - EntityChanges
///  - EndOfTick
#[derive(Debug, Clone, Serialize)]
#[qubit::ts]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TickEvent {
    /// A new tick has started
    StartOfTick { tick_id: TickId },

    /// A new tick has ended
    EndOfTick { tick_id: TickId },

    /// Set of changes to entities during the last tick
    EntityChanges { changes: Vec<EntityManagerMutation> },
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
    fn new(player_count: usize, preceding_player_id: Option<MatchId>) -> Self {
        Self {
            match_id: Uuid::now_v7().hyphenated().to_string(),
            player_count: player_count as i64,
            preceding_match_id: preceding_player_id,
            created_at: Local::now().naive_utc(),
        }
    }

    pub fn isolated(player_count: usize) -> Self {
        Self::new(player_count, None)
    }

    /// Get one match config from the db
    pub async fn get(db: &Db, match_id: MatchId) -> anyhow::Result<Self> {
        sqlx::query_file_as!(Self, "queries/get_match_config.sql", match_id)
            .fetch_one(db)
            .await
            .context("getting match config")
    }

    pub async fn save(&self, db: &Db) -> anyhow::Result<()> {
        info!("Saving match configuration {} to db", &self.match_id);
        sqlx::query_file_as!(
            Self,
            "queries/set_match_config.sql",
            self.match_id,
            self.player_count,
            self.preceding_match_id,
            self.created_at
        )
        .execute(db)
        .await
        .map(|_| ())
        .context("Saving match config")
    }
}
