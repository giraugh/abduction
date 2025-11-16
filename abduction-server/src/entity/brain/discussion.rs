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

/* Couple more notes on process

= TICK 1 =
-> Entity with `is_lead` considers leading actions
    - e.g
        - Greet()
       - AskOpinion()
       - AskForInfo()
       - AskPersonal()
    - DOING so emits an action during resolution
    - when they do so they UNSET the `is_lead` state

-> Entity w/o is_lead most likely takes the `Nothing` action (for now)

= TICK 2 =
    -> Entity without `is_lead` responds to the event
    - AnswerQuestion()
    - GetDistracted()
    - Greet()
    IN DOING SO they now get to set `is_lead` and take the turn

    -> Original talking entity no longer has `is_lead` so now they probably take the `Nothing` action
*/

use std::str::FromStr;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use tracing::warn;

use crate::{
    entity::{
        brain::{actor_action::ActorAction, focus::ActorFocus, ActorActionResult},
        Entity, EntityId,
    },
    logs::{GameLog, GameLogBody},
    mtch::ActionCtx,
};

/// Actor actions relevant only during the "Discussion" focus
///
/// All discussion actions reduce interest
/// the "lose interest" action increases the rate of that loss
/// however, most actions increase the bond which increases the interest in future convos
#[derive(Clone, Debug)]
pub enum DiscussionAction {
    /// Lose interest in the conversation
    LoseInterest,

    /// Lead actions
    /// only takeable when the `is_lead` is set
    Lead(DiscussionLeadAction),

    /// Responses to lead actions, only selected during an event responding to someone talking
    Respond(DiscussionRespondAction),
}

impl Into<ActorAction> for DiscussionAction {
    fn into(self) -> ActorAction {
        ActorAction::Discussion(self)
    }
}

/// Lead actions
/// only takeable when the `is_lead` is set
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Hash, strum::Display)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub enum DiscussionLeadAction {
    #[strum(to_string = "opinion:{0}")]
    AskOpinionOnEntity(EntityId),
    #[strum(to_string = "personal:{0}")]
    AskPersonal(PersonalTopic),
    #[strum(to_string = "info:{0}")]
    AskForInfo(InfoTopic),
}

impl FromStr for DiscussionLeadAction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (tag, rest) = s
            .split_once(":")
            .ok_or(anyhow!("Malformed discussion lead action. No tag"))?;
        match tag {
            "opinion" => Ok(DiscussionLeadAction::AskOpinionOnEntity(rest.parse()?)),
            "personal" => Ok(DiscussionLeadAction::AskPersonal(rest.parse()?)),
            "info" => Ok(DiscussionLeadAction::AskForInfo(rest.parse()?)),
            _ => Err(anyhow!(
                "Failed to parse discussion lead action, unkown tag {tag}"
            )),
        }
    }
}

/// An opinion on an entity
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[qubit::ts]
pub enum Opinion {
    Positive,
    #[default]
    Neutral,
    Negative,
}

/// Responses to lead actions
#[derive(Clone, Debug, Serialize, Deserialize)]
#[qubit::ts]
pub enum DiscussionRespondAction {
    /// Given an opinion on some entity
    /// derived from relations
    GiveOpinion(Opinion),

    /// Give an answer to some personal question
    /// (NOTE: this includes the resolved display of the answer)
    GivePersonal(PersonalTopic, String),

    /// Give some info (a meme) based on some question
    GiveInfo(InfoTopic),

    /// Refuse to answer a question because its too personal / rude
    /// (What this looks like may vary between entities / instances)
    Balk,
}

#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    Hash,
    strum::Display,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[qubit::ts]
pub enum PersonalTopic {
    Fear,
    Hope,
}

impl FromStr for PersonalTopic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for topic in PersonalTopic::VARIANTS {
            if topic.to_string() == s {
                return Ok(*topic);
            }
        }
        Err(anyhow!("No such personal topic {s:?}"))
    }
}

#[derive(
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    Hash,
    strum::Display,
    strum::VariantArray,
)]
#[serde(rename_all = "snake_case")]
#[qubit::ts]
pub enum InfoTopic {
    WaterSourceLocation,
    ShelterLocation,
}

impl FromStr for InfoTopic {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for topic in InfoTopic::VARIANTS {
            if topic.to_string() == s {
                return Ok(*topic);
            }
        }
        Err(anyhow!("No such info topic {s:?}"))
    }
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
            ref mut is_lead,
            with,
        }) = self.attributes.focus.as_mut()
        else {
            warn!("Attempted to resolve discussion action but not in a discussion");
            return ActorActionResult::NoEffect;
        };

        // if other is no longer in the discussion, we need to leave too
        // and discard whatever we were going to do
        let Some(interlocutor) = ctx.entities.by_id(with) else {
            warn!("Entity being discussed with does not exist");
            self.attributes.focus = Some(ActorFocus::Unfocused);
            return ActorActionResult::NoEffect;
        };
        match interlocutor.attributes.focus.as_ref() {
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
        // but more if we take the "lose interest" action
        let interest_loss = match action {
            DiscussionAction::LoseInterest => 3,
            _ => 1,
        };
        *interest = interest.saturating_sub(interest_loss);

        // if fully uninterested, leave the discussion
        // discarding what we do otherwise
        if *interest == 0 {
            self.attributes.focus = Some(ActorFocus::Unfocused);
            ctx.send_log(GameLog::entity_pair(
                self,
                interlocutor,
                GameLogBody::EntityFarewell,
            ));

            return ActorActionResult::NoEffect;
        }

        // Was this a lead action?
        if let DiscussionAction::Lead(lead_action) = action {
            // Remember we asked this so we dont do it again w/ this same interlocutor
            self.attributes
                .memes
                .as_mut()
                .unwrap()
                .remember_asked(with, lead_action);

            // We lose the `lead` status
            // the other interlocutor will respond and then become the new lead
            *is_lead = false;
        }

        // Emit a log about the thing we said/did
        match action {
            DiscussionAction::Lead(discussion_lead_action) => {
                ctx.send_log(GameLog::entity_pair(
                    self,
                    interlocutor,
                    GameLogBody::EntityAsk(discussion_lead_action.clone()),
                ));
            }
            DiscussionAction::Respond(discussion_respond_action) => {
                ctx.send_log(GameLog::entity_pair(
                    self,
                    interlocutor,
                    GameLogBody::EntityRespond(discussion_respond_action.clone()),
                ));
            }
            DiscussionAction::LoseInterest => {
                ctx.send_log(GameLog::entity_pair(
                    self,
                    interlocutor,
                    GameLogBody::EntityLoseInterest,
                ));
            }
        }

        ActorActionResult::Ok
    }
}
