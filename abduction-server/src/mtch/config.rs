use anyhow::Context;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use tracing::info;
use uuid::Uuid;

use crate::Db;

use super::MatchId;

/// The configuration for a given match
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
#[qubit::ts]
pub struct MatchConfig {
    /// Unique v7 uuid for this match
    pub match_id: MatchId,

    /// The number of players in the match
    ///  - Players will be copied across from predecessor match if appropriate,
    ///  - otherwise new players will be generated when the match is setup
    ///
    /// TODO: I really want this to be unsigned...
    pub player_count: i32,

    /// Optionally, the id of the match preceding
    /// this one. If set, players and some entities may be copied across
    /// #[sqlx(try_from = "Option<String>")]
    pub preceding_match_id: Option<MatchId>,

    /// How far the world extends in every direction as a number of hexs
    /// TODO: I really want this to be unsigned...
    pub world_radius: i32,
}

impl MatchConfig {
    fn new(player_count: usize, world_radius: usize, preceding_player_id: Option<MatchId>) -> Self {
        Self {
            match_id: Uuid::now_v7().hyphenated().to_string(),
            player_count: player_count as i32,
            preceding_match_id: preceding_player_id,
            world_radius: world_radius as i32,
        }
    }

    pub fn isolated(player_count: usize, world_extents: usize) -> Self {
        Self::new(player_count, world_extents, None)
    }

    /// Get one match config from the db
    #[allow(unused)]
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
            self.world_radius
        )
        .execute(db)
        .await
        .map(|_| ())
        .context("Saving match config")
    }
}
