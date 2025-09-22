//! Things that affect the whole world

use rand::{
    distr::{self, Distribution},
    Rng,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::logs::{GameLog, GameLogBody};

/// Describes current state of the world
#[derive(Debug, Clone, Serialize, Deserialize)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub struct EntityWorld {
    pub time_of_day: TimeOfDay,
    pub weather: WeatherKind,
    pub day: usize,
}

impl Default for EntityWorld {
    fn default() -> Self {
        Self {
            day: 1,
            time_of_day: TimeOfDay::default(),
            weather: WeatherKind::default(),
        }
    }
}

impl EntityWorld {
    pub fn update(&mut self, log_tx: &broadcast::Sender<GameLog>, rng: &mut impl Rng) {
        // Update TOD
        self.time_of_day = self.time_of_day.next();
        log_tx
            .send(GameLog::global(GameLogBody::TimeOfDayChange {
                time_of_day: self.time_of_day.clone(),
            }))
            .unwrap();

        // Go to next day
        if self.time_of_day == TimeOfDay::Morning {
            self.day += 1;
        }

        // Update weather
        if let Some(next_weather) = self.weather.next_weather(rng) {
            // logs
            self.weather = next_weather;
            log_tx
                .send(GameLog::global(GameLogBody::WeatherChange {
                    weather: self.weather.clone(),
                }))
                .unwrap();
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub enum TimeOfDay {
    #[default]
    Morning,
    Afternoon,
    Night,
}

impl TimeOfDay {
    pub fn next(&self) -> Self {
        match self {
            TimeOfDay::Morning => TimeOfDay::Afternoon,
            TimeOfDay::Afternoon => TimeOfDay::Night,
            TimeOfDay::Night => TimeOfDay::Morning,
        }
    }
}

impl TimeOfDay {
    /// Get the multiplier for a chance of a cold proc happening for the current time of day
    pub fn current_temp_as_cold_proc_chance_scale(&self) -> f32 {
        match self {
            TimeOfDay::Morning => 0.3,
            TimeOfDay::Afternoon => 0.0,
            TimeOfDay::Night => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[qubit::ts]
#[serde(rename_all = "snake_case")]
pub enum WeatherKind {
    /// Nice gently sun and clouds - no additional effects
    #[default]
    Lovely,

    /// Strong sun - things dry faster
    Sunny,

    /// Shadowy overcast (mostly just flavour)
    Overcast,

    /// Some light wind - increased chance of getting cold at night
    LightWind,

    /// Some heavy wind - increased chanced of getting cold at night
    Hurricane,

    /// Little bit of light rain - may cause some players to get wet
    LightRain,

    /// Large amount of rain - players will get quite wet
    HeavyRain,

    /// Stormy weather with heavy rain and lightning - players will get wet and random lightning strikes occur
    LightningStorm,
    // Earthquake
    // Tornado,
}

impl WeatherKind {
    pub fn rain_proc_chance_scale(&self) -> f32 {
        match self {
            WeatherKind::Lovely => 0.0,
            WeatherKind::Sunny => 0.0,
            WeatherKind::Overcast => 0.0,
            WeatherKind::LightWind => 0.0,
            WeatherKind::Hurricane => 0.0,
            WeatherKind::LightRain => 0.3,
            WeatherKind::HeavyRain => 1.0,
            WeatherKind::LightningStorm => 1.0,
        }
    }

    pub fn wind_proc_chance_scale(&self) -> f32 {
        match self {
            WeatherKind::Lovely => 0.0,
            WeatherKind::Sunny => 0.0,
            WeatherKind::Overcast => 0.2,
            WeatherKind::LightWind => 0.5,
            WeatherKind::Hurricane => 1.0,
            WeatherKind::LightRain => 0.4,
            WeatherKind::HeavyRain => 0.4,
            WeatherKind::LightningStorm => 0.9,
        }
    }

    pub fn transitions(&self) -> Vec<(Self, usize)> {
        use WeatherKind::*;
        match self {
            Lovely => vec![(Lovely, 5), (Overcast, 5), (LightWind, 2), (LightRain, 2)],
            Sunny => vec![(Sunny, 5), (Lovely, 5), (Overcast, 2)],
            Overcast => vec![(Overcast, 5), (Lovely, 5), (LightWind, 5), (LightRain, 5)],
            LightWind => vec![
                (LightWind, 5),
                (Overcast, 5),
                (Hurricane, 1),
                // (LightningStorm, 1),
                (LightRain, 2),
            ],
            Hurricane => vec![(Hurricane, 5), (LightWind, 5), (LightningStorm, 2)],
            LightRain => vec![(LightRain, 5), (HeavyRain, 4)],
            HeavyRain => vec![(HeavyRain, 5), (LightRain, 5), (LightningStorm, 2)],
            LightningStorm => vec![(LightningStorm, 5), (HeavyRain, 5)],
        }
    }

    /// Get the next weather to occur
    /// if the same weather happens again, returns None
    pub fn next_weather(&self, rng: &mut impl Rng) -> Option<Self> {
        let (weathers, weights): (Vec<_>, Vec<_>) = self.transitions().into_iter().unzip();
        let dist = distr::weighted::WeightedIndex::new(weights).unwrap();
        let next_index = dist.sample(rng);
        let next_weather = weathers[next_index].clone();
        if next_weather == *self {
            None
        } else {
            Some(next_weather)
        }
    }
}
