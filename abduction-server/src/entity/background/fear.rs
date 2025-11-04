use serde_repr::{Deserialize_repr, Serialize_repr};

/// Stylised like "fear of X"
/// NOTE: this is stored as an int to reduce memory use, then when needed its expanded to a string representation using `Display`.
///       i.e when used in a gamelog, the expanded version should be included
#[derive(Debug, Clone, strum::Display, Serialize_repr, Deserialize_repr, strum::VariantArray)]
#[repr(usize)]
#[qubit::ts]
#[ts(as = "usize")]
pub enum Fear {
    #[strum(to_string = "failing when they have responsibility")]
    Failure,

    #[strum(to_string = "being rejected")]
    Rejection,

    #[strum(to_string = "being abandoned")]
    Abandonment,

    #[strum(to_string = "being judged by others")]
    BeingJudged,

    #[strum(to_string = "being left alone")]
    Loneliness,

    #[strum(to_string = "being close and intimate with others")]
    Intimacy,

    #[strum(to_string = "losing people close to them")]
    Loss,

    #[strum(to_string = "things changing and not being able to keep up")]
    Change,

    #[strum(to_string = "being disappointed by the world and by others")]
    Disappointment,

    #[strum(to_string = "being vulnerable to others")]
    Vulnerability,

    #[strum(to_string = "being forgotten when they die")]
    BeingForgotten,

    #[strum(to_string = "succeeding when they dont think its deserved")]
    Success,

    #[strum(to_string = "embarassing themselves by trying too hard")]
    Embarrassment,

    #[strum(to_string = "not being good enough")]
    NotBeingGoodEnough,

    #[strum(to_string = "letting other people down")]
    LettingPeopleDown,

    #[strum(to_string = "making a severe mistake")]
    MakingMistakes,

    #[strum(to_string = "death")]
    Death,

    #[strum(to_string = "growing old")]
    Aging,

    #[strum(to_string = "getting sick as they age")]
    Illness,

    #[strum(to_string = "experiencing extreme pain")]
    Pain,

    #[strum(to_string = "the unknown")]
    TheUnknown,

    #[strum(to_string = "losing control of their life")]
    LosingControl,

    #[strum(to_string = "being trapped somewhere, unable to escape")]
    BeingTrapped,

    #[strum(to_string = "being betrayed by the people they trust")]
    Betrayal,

    #[strum(to_string = "failing to act when their help is needed")]
    FailureToAct,

    #[strum(to_string = "being fundamentally misunderstood by the people closest to them")]
    BeingMisunderstood,

    #[strum(to_string = "never making a real connection with another person before they die")]
    BeingAloneForever,

    #[strum(to_string = "being inadequate")]
    Inadequacy,

    #[strum(to_string = "being intellectually inferior to everyone around them")]
    IntellectualInferiority,

    #[strum(to_string = "being socially rejected")]
    SocialRejection,

    #[strum(to_string = "being responsible for others")]
    Responsibility,

    #[strum(to_string = "being criticised")]
    Criticism,

    #[strum(to_string = "being neglected and left to die")]
    Neglect,

    #[strum(to_string = "losing their closest loved ones")]
    LosingLovedOnes,

    #[strum(to_string = "needing to suffer through emotional pain")]
    EmotionalPain,

    #[strum(to_string = "not making an impact on the world before they die")]
    BeingForgottenByHistory,

    #[strum(to_string = "feeling like an outsider their whole lives")]
    NotFittingIn,

    #[strum(to_string = "finding out some important but disturbing truth")]
    ExposureToTruth,

    #[strum(to_string = "finding out that they've been lied to by someone close to them")]
    BeingLiedTo,

    #[strum(to_string = "introspecting their own thoughts and emotions")]
    Introspection,

    #[strum(to_string = "repeating the mistakes of their parents")]
    BecomingLikeParents,

    #[strum(to_string = "being manipulated by others")]
    Manipulation,

    #[strum(to_string = "failing at their relationships")]
    FailureInLove,

    #[strum(to_string = "being emotionally dependent on their friends and families")]
    EmotionalDependence,

    #[strum(to_string = "never fulfilling their potential")]
    UnfulfilledPotential,

    #[strum(to_string = "becoming irrelevant")]
    Irrelevance,

    #[strum(to_string = "never achieving a semblance of spiritual enlightenment")]
    SpiritualEmptiness,

    #[strum(to_string = "discovering their own meaninglessness")]
    Meaninglessness,

    #[strum(to_string = "running out of time")]
    TimeRunningOut,

    #[strum(to_string = "being judged poorly when they die")]
    BeingJudgedAfterDeath,

    #[strum(to_string = "their rocky past catching up to them")]
    PastCatchingUp,

    #[strum(to_string = "discovering what the future has in store for them")]
    FutureEvents,

    #[strum(to_string = "what new technology will bring")]
    Technology,

    #[strum(to_string = "machines gaining sentience")]
    ArtificialIntelligence,

    #[strum(to_string = "being in large crowds")]
    Crowds,

    #[strum(to_string = "silence")]
    Silence,

    #[strum(to_string = "darkness")]
    Darkness,

    #[strum(to_string = "large open spaces")]
    OpenSpaces,

    #[strum(to_string = "closed tight spaces")]
    EnclosedSpaces,

    #[strum(to_string = "tall heights")]
    Heights,

    #[strum(to_string = "falling from a great height")]
    Falling,

    #[strum(to_string = "drowning")]
    Drowning,

    #[strum(to_string = "being attacked by insects")]
    AttackedByInsects,

    #[strum(to_string = "being covered in insects")]
    CoveredByInsects,

    #[strum(to_string = "spiders")]
    Spiders,

    #[strum(to_string = "snakes")]
    Snakes,

    #[strum(to_string = "worms")]
    Worms,

    #[strum(to_string = "bats")]
    Bats,

    #[strum(to_string = "deep holes")]
    DeepHoles,

    #[strum(to_string = "being outside")]
    Outdoors,

    #[strum(to_string = "birds attacking them")]
    Birds,

    #[strum(to_string = "seeing another persons blood")]
    Blood,

    #[strum(to_string = "failing to protect the people they love")]
    FailureToProtect,

    #[strum(to_string = "causing their own destruction")]
    SelfDestruction,

    #[strum(to_string = "losing their sanity")]
    Insanity,

    #[strum(to_string = "seeing something terrifying in their dreams")]
    Dreams,

    #[strum(to_string = "never making it back to see their family")]
    NotMakingItBack,

    #[strum(to_string = "paranormal spirits haunting them")]
    BeingHaunted,

    #[strum(to_string = "being controlled by devils")]
    Devils,
}
