use serde::{de::Visitor, Deserialize, Serialize};
use strum::VariantArray;

#[derive(Debug, Clone, strum::VariantArray, strum::IntoStaticStr)]
#[allow(clippy::enum_variant_names)]
#[qubit::ts]
pub enum Career {
    // technical / software / industrial
    SoftwareEngineer,
    DataScientist,
    MachineLearningEngineer,
    AIResearcher,
    UXDesigner,
    GraphicDesigner,
    VideoGameDeveloper,
    WebDeveloper,
    MobileAppDeveloper,
    CloudArchitect,
    DevOpsEngineer,
    CybersecurityAnalyst,
    EthicalHacker,
    SystemsAdministrator,
    NetworkEngineer,
    DatabaseAdministrator,
    SoftwareTester,
    GameTester,
    RoboticsEngineer,
    HardwareEngineer,
    IoTEngineer,
    AutomationEngineer,
    DataEngineer,
    BigDataArchitect,
    RoboticsTechnician,
    CybersecurityEngineer,
    SoftwarePenetrationTester,
    EmbeddedSystemsDeveloper,
    HardwareHacker,
    LogisticsManager,
    SupplyChainSpecialist,
    SoundEngineer,
    UrbanPlanner,
    CivilEngineer,
    MechanicalEngineer,
    ElectricalEngineer,
    AerospaceEngineer,
    IndustrialDesigner,
    HydrologyEngineer,
    SimulationEngineer,
    RenewableEnergyEngineer,
    SolarEngineer,
    WindEnergySpecialist,

    // media / art / writing / makes stuff
    Animator,
    Illustrator,
    TechnicalWriter,
    ContentStrategist,
    DigitalMarketer,
    Copywriter,
    Journalist,
    Editor,
    Author,
    Screenwriter,
    Playwright,
    Poet,
    Architect,
    InteriorDesigner,
    TattooArtist,
    GameDesigner,
    LevelDesigner,
    UXResearcher,
    StageDirector,
    FilmDirector,
    Producer,
    Cinematographer,
    Photographer,
    FashionDesigner,
    JewelryDesigner,
    ProductDesigner,
    MakeupArtist,
    HairStylist,
    Cosmetologist,
    Potter,
    Artist,
    Painter,
    DigitalArtist,
    ConceptArtist,
    StoryboardArtist,
    Writer,
    Blogger,
    YouTuber,
    MusicProducer,
    PodcastProducer,
    Composer,
    Musician,
    Choreographer,
    FilmEditor,
    PostProductionSpecialist,
    SpecialEffectsArtist,
    LightingTechnician,
    StageManager,
    SetDesigner,
    PropDesigner,
    CostumeDesigner,
    RadioProducer,
    TechBlogger,
    EnvironmentalJournalist,
    DocumentaryFilmmaker,
    ScientificIllustrator,
    Curator,
    ContentCreator,

    // craftsperson
    Craftsman,
    Metalworker,
    Blacksmith,
    Glassblower,
    Sculptor,
    Woodworker,

    // acting / presenting
    Actor,
    VoiceActor,
    Streamer,
    Comedian,
    Magician,
    Illusionist,
    StreetPerformer,
    Singer,
    DiskJockey,
    RadioHost,
    Podcaster,
    Influencer,
    ScienceCommunicator,
    MotivationalSpeaker,
    FashionModel,

    // consultancy?
    SocialMediaManager,
    BrandManager,
    CastingDirector,
    TalentAgent,

    // mental health
    Therapist,
    Counselor,
    SocialWorker,
    Physiotherapist,
    OccupationalTherapist,
    SpeechPathologist,
    Psychotherapist,
    ClinicalPsychologist,
    SocialPsychologist,
    DevelopmentalPsychologist,
    OccupationalPsychologist,
    LifeStrategist,
    MusicTherapist,

    // pedagogic
    EnglishTeacher,
    MathTeacher,
    ScienceTeacher,
    Professor,
    Tutor,
    YogaInstructor,
    Philosopher,
    Clergy,
    SpiritualGuide,
    YogaTeacher,
    MeditationInstructor,
    EsportsCoach,
    DanceInstructor,
    VoiceCoach,

    // science / researcher
    AcademicResearcher,
    Scientist,
    Chemist,
    Biologist,
    Physicist,
    Mathematician,
    Astronomer,
    Ecologist,
    Geologist,
    Psychologist,
    ResearchScientist,
    Linguist,
    LabTechnician,
    ForensicAnalyst,
    Bioinformatician,
    Geneticist,
    Neuroscientist,
    CognitiveScientist,
    ResearchAssistant,
    LabManager,
    PharmaceuticalResearcher,
    ClimateScientist,
    MathematicalModeler,
    Historian,

    // medical
    Nurse,
    Doctor,
    Surgeon,
    Dentist,
    Pharmacist,
    Paramedic,
    HealthcareConsultant,
    PublicHealthSpecialist,
    Epidemiologist,
    NutritionScientist,

    // works with animals
    Veterinarian,
    DogWalker,
    AnimalGroomer,
    AnimalTrainer,

    // athletic / physical
    AthleticTrainer,
    ProfessionalAthlete,
    PersonalTrainer,

    // food / drink preparation
    Nutritionist,
    Chef,
    SousChef,
    Baker,
    Barista,
    Sommelier,

    // manager / interface with people
    RestaurantManager,
    HospitalityManager,
    ProductManager,
    EventPlanner,
    EventCoordinator,
    InfluencerManager,
    HRManager,
    TeamLead,
    RealEstateAgent,
    Lawyer,
    Paralegal,
    Judge,
    Mediator,
    Politician,
    Diplomat,
    Activist,
    NonProfitManager,
    CommunityOrganizer,

    // vehicles
    TravelAgent,
    FlightAttendant,
    AirTrafficController,
    Pilot,

    // finance
    Entrepreneur,
    StartupFounder,
    Investor,
    VentureCapitalist,
    FinancialPlanner,
    EstatePlanner,
    InsuranceAdvisor,
    CareerCounselor,
    Recruiter,
    BusinessAnalyst,
    FinancialAnalyst,
    Accountant,
    Economist,
    Auditor,

    // works in/with nature
    Conservationist,
    ParkRanger,
    MarineBiologist,
    Zoologist,
    Astronaut,
    Explorer,
    TravelWriter,
    AdventureGuide,
    MountaineeringInstructor,
    SailingInstructor,
    ScubaInstructor,
    WildlifePhotographer,
    Oceanographer,
    MarineEcologist,
    FisheriesScientist,
    AquacultureSpecialist,

    // military / police / defence
    MilitaryOfficer,
    Detective,
    PoliceOfficer,
    DronePilot,
    Firefighter,
    SecurityConsultant,

    // mapping / spatial
    GeospatialAnalyst,
    Cartographer,
    GISSpecialist,
    RemoteSensingAnalyst,
    DroneSurveyor,

    Barber,
    Inventor,
    IntelligenceAnalyst,
    EnvironmentalConsultant,
    SustainabilityConsultant,
    Translator,
    Interpreter,
    Archivist,
    MuseumDirector,
    UrbanFarmer,
    BloggerManager,
    MotivationalCoach,
    HealthCoach,
    PrivacyConsultant,
    DigitalForensicsAnalyst,
    ClinicalResearchCoordinator,
    EnvironmentalHealthOfficer,
    SustainabilityAnalyst,
    UrbanEcologist,
    EnvironmentalPolicyAnalyst,
    WildlifeConservationist,
}

fn lower_with_spaces(s: String) -> String {
    s.chars()
        .enumerate()
        .map(|(i, c)| {
            if i > 0 && c.is_uppercase() {
                format!(" {}", c.to_lowercase())
            } else {
                c.to_ascii_lowercase().to_string()
            }
        })
        .collect::<String>()
}

impl ToString for Career {
    fn to_string(&self) -> String {
        let discriminator: &'static str = self.into();
        lower_with_spaces(discriminator.to_string())
    }
}

/// I really don't like doing this every time we serialize :(
/// would love to just have this be a static lookup...
/// TODO: improve all this
impl Serialize for Career {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let display = format!("{self:?}");
        let with_spaces = lower_with_spaces(display);
        serializer.serialize_str(&with_spaces)
    }
}

/// This is inefficiently implemented but it should happen rarely
impl<'de> Deserialize<'de> for Career {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CareerVisitor;

        impl<'de> Visitor<'de> for CareerVisitor {
            type Value = Career;

            fn expecting(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
                fmt.write_str("a career name in lowercase with spaces")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // Consider each variant
                for variant in Career::VARIANTS.iter() {
                    let discriminator: &'static str = variant.into();
                    let with_spaces = lower_with_spaces(discriminator.to_string());
                    if with_spaces == v {
                        return Ok(variant.to_owned());
                    }
                }

                Err(E::custom(format!("unknown career: {v}")))
            }
        }

        deserializer.deserialize_str(CareerVisitor)
    }
}
