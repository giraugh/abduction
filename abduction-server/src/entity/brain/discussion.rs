use rand::seq::IndexedRandom;
use serde::Serialize;
use tracing::warn;

use crate::{
    entity::{
        brain::{focus::ActorFocus, motivator, ActorActionResult},
        Entity,
    },
    logs::{GameLog, GameLogBody},
    mtch::ActionCtx,
};

/*
== Some quick ideation ==

John waves at Smith
Smith waves back
John beckons smith over
John and smith sit down to chat                                        -> Turn order kind of thing
John asks Smith what they think about the weather                      -> AskOpinion<>
Smith answers <thoughtfully> that they like it                         -> random variants
Smith asks John if they've seen any good sources of fresh water        -> Inquire about missing memes
John's stomach rumbles
John gets distracted for a second
Smith frowns in frustration and attempts to get johns attention back   -> Smith's patience characteristic is low
Smith asks John what they think of <Ella>
John responds that he is fond of them                                  -> Increases opinion of someone else, scaled by opinion of interlocutor
John stands up and says farewell to Smith
Smith farewells John

==

John waves at Smith
Smith waves back
John beckons smith over
John and smith sit down to chat
John asks Smith a deeply personal question about <their mother>        -> Low social awareness?
Smith is taken aback and refuses to answer                             -> Bond is too low
With a frown, Smith stands up and farewells John
John does not answer

==

John waves at Smith
Smith does not wave back
Smith winds up a punch, aiming at John

==

Things to talk about
 - ask opinion
   - weather
   - another player
 - ask for info
   - sources of water
   - sources of shelter
 - ask personal question
   - about a parent / sibiling
   - about their fears
   - about their hopes
*/

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
        ctx: &ActionCtx,
    ) -> ActorActionResult {
        // Get a reference to the discussion focus
        let Some(ActorFocus::Discussion {
            ref mut interest,
            with,
        }) = self.attributes.focus.as_mut()
        else {
            warn!("Attempted to resolve discussion action but not in a discussion");
            return ActorActionResult::NoEffect;
        };

        // if other is no longer in the discussion, we need to leave too
        // and discard whatever we were going to do
        let Some(with_entity) = ctx.entities.by_id(with) else {
            warn!("Entity being discussed with does not exist");
            self.attributes.focus = Some(ActorFocus::Unfocused);
            return ActorActionResult::NoEffect;
        };
        match with_entity.attributes.focus.as_ref() {
            // Talking with us?
            Some(ActorFocus::Discussion {
                with: other_with, ..
            }) if other_with == &self.entity_id => {}

            // Not talking / not with us?
            _ => {
                self.attributes.focus = Some(ActorFocus::Unfocused);
                // NOTE: don't do a log because its no-longer guaranteed we are still near them
                return ActorActionResult::NoEffect;
            }
        }

        // Always lose interest
        *interest = interest.saturating_sub(1);

        // if fully uninterested, leave the discussion
        // discarding what we do otherwise
        if *interest == 0 {
            self.attributes.focus = Some(ActorFocus::Unfocused);
            ctx.send_log(GameLog::entity_pair(
                self,
                with_entity,
                GameLogBody::EntityFarewell,
            ));

            return ActorActionResult::NoEffect;
        }

        // Now actually resolve the action
        match action {
            DiscussionAction::LoseInterest => {
                // Lose extra interest (x3)
                *interest = interest.saturating_sub(2);

                // send log
                ctx.send_log(GameLog::entity_pair(
                    self,
                    with_entity,
                    GameLogBody::EntityLoseInterest,
                ))
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
                ctx.send_log(GameLog::entity_pair(
                    self,
                    with_entity,
                    GameLogBody::EntityChat {
                        topic: topic.clone(),
                    },
                ));
            }
        }

        ActorActionResult::NoEffect
    }
}
