pub mod config;
pub use config::*;

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
use rand::Rng;
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::info;

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
    match_config: MatchConfig,
    match_entities: EntityManager,
}

impl MatchManager {
    pub async fn load_match(match_config: MatchConfig, db: &Db) -> Self {
        // Create an entity manager and load the entities for the match
        let mut match_entities = EntityManager::new(&match_config.match_id);
        match_entities.load_entities(db).await;

        Self {
            match_config,
            match_entities,
        }
    }

    /// Load in a match configuration, generating any resources needed for the game
    ///
    /// This should only be done once per match, realistically - so prob do it when
    /// the config is created
    ///
    /// NOTE: This can be done before the match is ready to be run
    ///       i.e right after the previous match if appropriate.
    pub async fn initialise_new_match(&mut self, db: &Db) -> anyhow::Result<()> {
        // Now we initialise it...
        info!("Initialising match {}", &self.match_config.match_id);

        // Add all the unescaped players from the last game
        // In practice, this just means cloning the entity into the new match
        let mut existing_players = 0;
        if let Some(preceding_match_id) = &self.match_config.preceding_match_id {
            EntityManager::load_entities_from_match(preceding_match_id, db)
                .await
                .filter(|e| has_markers!(e, Player) && !has_markers!(e, Escaped))
                .for_each(|entity| {
                    existing_players += 1;
                    unimplemented!(); // TODO: actually copy across the entity
                })
        }

        // If we dont have enough players for the match configuration,
        // then generate and add more
        let player_count_to_gen = self.match_config.player_count - existing_players;
        for _ in 0..player_count_to_gen {
            let player_entity = generate_player()?;
            self.match_entities.upsert_entity(player_entity)?;
        }

        // Put players in the desired locations
        // TODO

        Ok(())
    }

    /// Perform one game tick
    /// When a match is on, this is called every second or so to update the state of the world
    pub async fn perform_match_tick(&mut self, tick_tx: &broadcast::Sender<TickEvent>, db: &Db) {
        // TODO: implement actual actions, agents, world changes etc

        // TEMP: for now just choose a random entity, and move it to the right
        {
            let random_entity_id = {
                let all_ids: Vec<_> = self
                    .match_entities
                    .get_all_entities()
                    .map(|e| &e.entity_id)
                    .collect();
                let index = rand::rng().random_range(0..all_ids.len());
                all_ids[index].clone()
            };

            self.match_entities
                .mutate(&random_entity_id, |e| {
                    if let Some(h) = e.attributes.hex.as_mut() {
                        h.0 += 1;
                    }
                })
                .unwrap();
        }

        // Flush changes to entities to the DB and to clients
        self.match_entities
            .flush_changes(tick_tx, db)
            .await
            .unwrap();
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
