use rand::seq::IndexedRandom;
use serde::Serialize;
use tokio::sync::broadcast;
use tracing::warn;

use crate::{
    entity::{
        brain::{focus::PlayerFocus, motivator, PlayerActionResult},
        Entity,
    },
    logs::{GameLog, GameLogBody},
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
    LightChat,

    /// Proper chat about themselves
    HeavyChat,
    //
    // TODO:
    //  - Insult / something rude
    //  - Share some information on location
    //  - Share some information on another player
}

#[derive(Clone, Debug, Serialize)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub enum Topic {
    // -- Low involvement --
    Career,
    Entertainment,
    News,
    AlienSituation,
    Weather,

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
        Topic::Weather,
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
        // Get a reference to the discussion focus
        let Some(PlayerFocus::Discussion {
            ref mut interest,
            with,
        }) = self.attributes.focus.as_mut()
        else {
            warn!("Attempted to resolve discussion action but not in a discussion");
            return PlayerActionResult::NoEffect;
        };

        // if other is no longer in the discussion, we need to leave too
        // and discard whatever we were going to do
        let Some(with_entity) = all_entities.iter().find(|e| e.entity_id == *with) else {
            warn!("Entity being discussed with does not exist");
            self.attributes.focus = Some(PlayerFocus::Unfocused);
            return PlayerActionResult::NoEffect;
        };
        match with_entity.attributes.focus.as_ref() {
            // Talking with us?
            Some(PlayerFocus::Discussion {
                with: other_with, ..
            }) if other_with == &self.entity_id => {}

            // Not talking / not with us?
            _ => {
                self.attributes.focus = Some(PlayerFocus::Unfocused);
                // NOTE: don't do a log because its no-longer guaranteed we are still near them
                return PlayerActionResult::NoEffect;
            }
        }

        // Always lose interest
        *interest = interest.saturating_sub(1);

        // if fully uninterested, leave the discussion
        // discarding what we do otherwise
        if *interest == 0 {
            self.attributes.focus = Some(PlayerFocus::Unfocused);
            log_tx
                .send(GameLog::entity_pair(
                    self,
                    with_entity,
                    GameLogBody::EntityFarewell,
                ))
                .unwrap();

            return PlayerActionResult::NoEffect;
        }

        // Now actually resolve the action
        match action {
            DiscussionAction::LoseInterest => {
                // Lose extra interest (x3)
                *interest = interest.saturating_sub(2);

                // send log
                log_tx
                    .send(GameLog::entity_pair(
                        self,
                        with_entity,
                        GameLogBody::EntityLoseInterest,
                    ))
                    .unwrap();
            }
            DiscussionAction::LightChat | DiscussionAction::HeavyChat => {
                // Determine topic
                let topic_set = match action {
                    DiscussionAction::LightChat => Topic::LIGHT_TOPICS,
                    DiscussionAction::HeavyChat => Topic::HEAVY_TOPICS,
                    _ => unreachable!(),
                };
                let mut rng = rand::rng();
                let topic = topic_set.choose(&mut rng).unwrap();

                // We get less sad
                self.attributes.motivators.reduce::<motivator::Sadness>();

                // And like them more
                self.relations.increase_associate_bond(with);

                // send log
                log_tx
                    .send(GameLog::entity_pair(
                        self,
                        with_entity,
                        GameLogBody::EntityChat {
                            topic: topic.clone(),
                        },
                    ))
                    .unwrap();
            }
        }

        PlayerActionResult::NoEffect
    }
}
