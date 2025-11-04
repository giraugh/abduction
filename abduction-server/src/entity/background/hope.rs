use serde_repr::{Deserialize_repr, Serialize_repr};

/// Stylised like "hope that X"
/// NOTE: this is stored as an int to reduce memory use, then when needed its expanded to a string representation using `Display`.
///       i.e when used in a gamelog, the expanded version should be included
#[derive(Debug, Clone, strum::Display, Serialize_repr, Deserialize_repr, strum::VariantArray)]
#[repr(usize)]
#[qubit::ts]
#[ts(as = "usize")]
pub enum Hope {
    #[strum(to_string = "they can find love")]
    Love,

    #[strum(to_string = "they can make new personal connections")]
    Connection,

    #[strum(to_string = "they can find belonging")]
    Belonging,

    #[strum(to_string = "they can find peace of mind")]
    PeaceOfMind,

    #[strum(to_string = "they can find new self-understanding")]
    Understanding,

    #[strum(to_string = "they can find a community which accepts them")]
    Acceptance,

    #[strum(to_string = "they can find forgiveness")]
    Forgiveness,

    #[strum(to_string = "they can be spiritually healed")]
    Healing,

    #[strum(to_string = "they can grow to be the person they need to be")]
    Growth,

    #[strum(to_string = "they can find redemption")]
    Redemption,

    #[strum(to_string = "they can find reconciliation")]
    Reconciliation,

    #[strum(to_string = "they can be happy")]
    Happiness,

    #[strum(to_string = "they can find a new purpose")]
    Purpose,

    #[strum(to_string = "they can find meaning")]
    Meaning,

    #[strum(to_string = "they can find meaning")]
    Fulfillment,

    #[strum(to_string = "they can be free again")]
    Freedom,

    #[strum(to_string = "they can find clarity")]
    Clarity,

    #[strum(to_string = "they can be stable again")]
    Stability,

    #[strum(to_string = "they can discover something unique")]
    Discovery,

    #[strum(to_string = "they can help to inspire the next generation")]
    Inspiration,

    #[strum(to_string = "they can find new confidence")]
    Confidence,

    #[strum(to_string = "they can learn to trust themselves")]
    SelfTrust,

    #[strum(to_string = "they can find courage")]
    Courage,

    #[strum(to_string = "they can become a more kind person")]
    Kindness,

    #[strum(to_string = "people will treat each other more kindly in the future")]
    WorldKindness,

    #[strum(to_string = "they can become a more compassionate person")]
    Compassion,

    #[strum(to_string = "people can learn to treat each other with more compassion")]
    WorldCompassion,

    #[strum(to_string = "they grow wise over time")]
    Wisdom,

    #[strum(to_string = "they can learn to be patient")]
    Patience,

    #[strum(to_string = "others can learn to be patient with them")]
    OthersPatience,

    #[strum(to_string = "they can find inner balance")]
    Balance,

    #[strum(to_string = "they can make peace with their past")]
    PeaceWithPast,

    #[strum(to_string = "they can make peace with whatever comes")]
    PeaceWithFuture,

    #[strum(to_string = "they can achieve inner strength")]
    InnerStrength,

    #[strum(to_string = "they can find simplicity")]
    Simplicity,

    #[strum(to_string = "they can find a safe place to just exist")]
    Safety,

    #[strum(to_string = "their friends and family will stay loyal to them")]
    Loyalty,

    #[strum(to_string = "they can learn to be more honest with others")]
    Honesty,

    #[strum(to_string = "they can find intimacy with another")]
    Intimacy,

    #[strum(to_string = "everyone can learn to respect each other and their needs")]
    WorldMutualRespect,

    #[strum(to_string = "they find long lasting love with another")]
    LongLastingLove,

    #[strum(to_string = "they can make a new start soon")]
    NewBeginnings,

    #[strum(to_string = "they get a second chance")]
    SecondChances,

    #[strum(to_string = "there are better days yet to come")]
    BetterDays,

    #[strum(to_string = "they can change")]
    Change,

    #[strum(to_string = "they can use their creativity to weather the storm")]
    Creativity,

    #[strum(to_string = "others will recognise their contributions")]
    Recognition,

    #[strum(to_string = "they can learn to accept themselves")]
    SelfAcceptance,

    #[strum(to_string = "they can improve themselves")]
    SelfImprovement,

    #[strum(to_string = "one day they will forgive themselves")]
    ForgivingMyself,

    #[strum(to_string = "they can let go of the past")]
    LettingGo,

    #[strum(to_string = "they can find peace in this chaos")]
    PeaceInChaos,

    #[strum(to_string = "they can get closer to their loved ones")]
    Closeness,

    #[strum(to_string = "they can learn to believe in themselves")]
    BeliefInMyself,

    #[strum(to_string = "they can build a supportive community")]
    Community,

    #[strum(to_string = "their trust is well-placed")]
    Trust,

    #[strum(to_string = "one day they can live in emotional safety")]
    EmotionalSafety,

    #[strum(to_string = "their playfulness is not suppressed by others")]
    Playfulness,

    #[strum(to_string = "they can see more of the world")]
    SeeTheWorld,

    #[strum(to_string = "their innocence is preserved")]
    Innocence,

    #[strum(to_string = "their faith in humanity is well-placed")]
    FaithInHumanity,

    #[strum(to_string = "the world can learn to be more generous")]
    Generosity,

    #[strum(to_string = "they get the opportunity to show gratitude more often")]
    Gratitude,

    #[strum(to_string = "they can again experience joy in the small things")]
    JoyInSmallThings,

    #[strum(to_string = "they can return to somewhere cozy in the future")]
    CozyPlace,

    #[strum(to_string = "they can establish a connection with nature")]
    ConnectionWithNature,

    #[strum(to_string = "they can establish harmony within their family once they return")]
    FamilyHarmony,

    #[strum(to_string = "they can make a new friend")]
    Friendship,

    #[strum(to_string = "their love is truly unconditional")]
    UnconditionalLove,

    #[strum(to_string = "they can remain resilient in the face of danger")]
    Resilience,

    #[strum(to_string = "this all ends peacefully")]
    PeacefulEndings,

    #[strum(to_string = "they can reinvent themselves once they return")]
    Reinvention,

    #[strum(to_string = "they learn from their mistakes")]
    LearningFromMistakes,

    #[strum(to_string = "there are gentler days on the horizon")]
    GentleDays,

    #[strum(to_string = "a time will come when they no longer have to fear")]
    LessFear,

    #[strum(to_string = "they can achieve inner peace")]
    InnerPeace,

    #[strum(to_string = "soon they will be able to let go of their pain")]
    LettingGoOfPain,

    #[strum(to_string = "they talking honestly with others is the right decision")]
    HonestConversations,

    #[strum(to_string = "they can get the rest they need to survive")]
    Rest,

    #[strum(to_string = "they are able to recover from this")]
    Recovery,

    #[strum(to_string = "their independence is preserved")]
    PreserveIndependence,

    #[strum(to_string = "they can make a difference to other peoples survival")]
    MakingADifference,

    #[strum(to_string = "they are able to leave a legacy")]
    LeavingALegacy,

    #[strum(to_string = "everyone can experience some joy, even when its challenging")]
    JoyWithoutReason,
}
