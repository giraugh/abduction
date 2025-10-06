use tokio::sync::broadcast;

use crate::{
    entity::{brain::PlayerActionResult, Entity},
    logs::GameLog,
    mtch::MatchConfig,
};

/// Actions relevant only during the "Discussion" focus
///
/// All discussion actions reduce interest
/// the "lose interest" action increases the rate of that loss
/// however, most actions increase the bond which increases the interest in future convos
#[derive(Clone, Debug)]
pub enum DiscussionAction {
    /// Lose interest in the conversation
    LoseInterest,

    /// Small talk about nothing
    Chat,

    /// Discuss some topic
    Topic(Topic),
    //
    // TODO:
    //  - Insult / something rude
    //  - Share some information on location
    //  - Share some information on another player
}

#[derive(Clone, Debug)]
pub enum Topic {
    // -- Low involvement --
    Career,
    Entertainment,
    News,
    AlienSituation,

    // -- High Involvement --
    Family,
    Fears,
    Ambitions,
    Hopes,
}

impl Topic {
    pub const LIGHT_TOPICS: &[Topic] = &[
        Topic::Career,
        Topic::Entertainment,
        Topic::News,
        Topic::AlienSituation,
    ];

    pub const HEAVY_TOPICS: &[Topic] =
        &[Topic::Family, Topic::Fears, Topic::Ambitions, Topic::Hopes];
}

impl Entity {
    pub fn resolve_discussion_action(
        &mut self,
        action: &DiscussionAction,
        all_entities: &Vec<Entity>,
        config: &MatchConfig,
        log_tx: &broadcast::Sender<GameLog>,
    ) -> PlayerActionResult {
        // TODO
        PlayerActionResult::NoEffect
    }
}
