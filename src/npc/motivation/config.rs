use std::{fs, path::Path};

use bevy::prelude::*;
use serde::Deserialize;

const CONFIG_PATH: &str = "config/motivation.toml";

#[derive(Debug, Clone, Deserialize, Default)]
struct RawMotivationConfig {
    #[serde(default)]
    defaults: RawDefaults,
    #[serde(default)]
    decay: RawDecay,
    #[serde(default)]
    gains: RawGains,
    #[serde(default)]
    dependency: RawDependency,
    #[serde(default)]
    mood_thresholds: RawMoodThresholds,
    #[serde(default)]
    alcohol: RawAlcohol,
    #[serde(default)]
    leisure: RawLeisure,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawDefaults {
    min: f32,
    max: f32,
    start: f32,
}

impl Default for RawDefaults {
    fn default() -> Self {
        Self {
            min: 0.0,
            max: 100.0,
            start: 60.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawDecay {
    per_second: f32,
}

impl Default for RawDecay {
    fn default() -> Self {
        Self { per_second: 0.25 }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawGains {
    task: f32,
    social: f32,
    leisure: f32,
}

impl Default for RawGains {
    fn default() -> Self {
        Self {
            task: 8.0,
            social: 6.0,
            leisure: 5.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawDependency {
    satisfaction_bonus: f32,
    deficit_penalty: f32,
}

impl Default for RawDependency {
    fn default() -> Self {
        Self {
            satisfaction_bonus: 4.0,
            deficit_penalty: 7.5,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawMoodThresholds {
    energised: f32,
    content: f32,
    tired: f32,
}

impl Default for RawMoodThresholds {
    fn default() -> Self {
        Self {
            energised: 80.0,
            content: 55.0,
            tired: 30.0,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawAlcohol {
    boost: f32,
    intoxication_seconds: f32,
    hangover_penalty: f32,
    hangover_decay_multiplier: f32,
    hangover_duration_seconds: f32,
    quality_penalty: f32,
    trigger_keywords: Vec<String>,
}

impl Default for RawAlcohol {
    fn default() -> Self {
        Self {
            boost: 12.0,
            intoxication_seconds: 90.0,
            hangover_penalty: 15.0,
            hangover_decay_multiplier: 1.6,
            hangover_duration_seconds: 180.0,
            quality_penalty: 0.2,
            trigger_keywords: vec![
                "tavern".to_string(),
                "ale".to_string(),
                "mead".to_string(),
                "wine".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawLeisure {
    keywords: Vec<String>,
}

impl Default for RawLeisure {
    fn default() -> Self {
        Self {
            keywords: vec![
                "supper".to_string(),
                "stories".to_string(),
                "lute".to_string(),
                "rest".to_string(),
                "tavern".to_string(),
            ],
        }
    }
}

/// Runtime configuration derived from `config/motivation.toml`.
#[derive(Resource, Debug, Clone)]
pub struct MotivationConfig {
    pub defaults: MotivationDefaults,
    pub gains: MotivationGains,
    pub decay: MotivationDecay,
    pub dependency: DependencyImpactConfig,
    pub thresholds: MotivationMoodThresholds,
    pub alcohol: AlcoholConfig,
    pub leisure: LeisureConfig,
}

#[derive(Debug, Clone)]
pub struct MotivationDefaults {
    pub min: f32,
    pub max: f32,
    pub start: f32,
}

#[derive(Debug, Clone)]
pub struct MotivationGains {
    pub task: f32,
    pub social: f32,
    pub leisure: f32,
}

#[derive(Debug, Clone)]
pub struct MotivationDecay {
    pub per_second: f32,
}

#[derive(Debug, Clone)]
pub struct DependencyImpactConfig {
    pub satisfaction_bonus: f32,
    pub deficit_penalty: f32,
}

#[derive(Debug, Clone)]
pub struct MotivationMoodThresholds {
    pub energised: f32,
    pub content: f32,
    pub tired: f32,
}

#[derive(Debug, Clone)]
pub struct AlcoholConfig {
    pub boost: f32,
    pub intoxication_seconds: f32,
    pub hangover_penalty: f32,
    pub hangover_decay_multiplier: f32,
    pub hangover_duration_seconds: f32,
    pub quality_penalty: f32,
    pub trigger_keywords: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct LeisureConfig {
    pub keywords: Vec<String>,
}

impl MotivationConfig {
    pub fn load_or_default() -> Self {
        let path = Path::new(CONFIG_PATH);
        match fs::read_to_string(path) {
            Ok(raw) => match toml::from_str::<RawMotivationConfig>(&raw) {
                Ok(parsed) => parsed.into(),
                Err(err) => {
                    warn!(
                        "Failed to parse {} ({}). Falling back to defaults.",
                        CONFIG_PATH, err
                    );
                    RawMotivationConfig::default().into()
                }
            },
            Err(err) => {
                warn!(
                    "Failed to read {} ({}). Falling back to defaults.",
                    CONFIG_PATH, err
                );
                RawMotivationConfig::default().into()
            }
        }
    }
}

impl From<RawMotivationConfig> for MotivationConfig {
    fn from(value: RawMotivationConfig) -> Self {
        let defaults = MotivationDefaults {
            min: value.defaults.min.min(value.defaults.max),
            max: value.defaults.max.max(value.defaults.min + f32::EPSILON),
            start: value
                .defaults
                .start
                .clamp(value.defaults.min, value.defaults.max),
        };

        let decay = MotivationDecay {
            per_second: value.decay.per_second.max(0.0),
        };

        let gains = MotivationGains {
            task: value.gains.task.max(0.0),
            social: value.gains.social.max(0.0),
            leisure: value.gains.leisure.max(0.0),
        };

        let dependency = DependencyImpactConfig {
            satisfaction_bonus: value.dependency.satisfaction_bonus.max(0.0),
            deficit_penalty: value.dependency.deficit_penalty.max(0.0),
        };

        let mut thresholds = MotivationMoodThresholds {
            energised: value.mood_thresholds.energised,
            content: value.mood_thresholds.content,
            tired: value.mood_thresholds.tired,
        };
        if thresholds.energised < thresholds.content {
            thresholds.energised = thresholds.content;
        }
        if thresholds.content < thresholds.tired {
            thresholds.content = thresholds.tired;
        }

        let alcohol = AlcoholConfig {
            boost: value.alcohol.boost.max(0.0),
            intoxication_seconds: value.alcohol.intoxication_seconds.max(0.0),
            hangover_penalty: value.alcohol.hangover_penalty.max(0.0),
            hangover_decay_multiplier: value.alcohol.hangover_decay_multiplier.max(1.0),
            hangover_duration_seconds: value.alcohol.hangover_duration_seconds.max(0.0),
            quality_penalty: value.alcohol.quality_penalty.clamp(0.0, 1.0),
            trigger_keywords: normalise_keywords(&value.alcohol.trigger_keywords),
        };

        let leisure = LeisureConfig {
            keywords: normalise_keywords(&value.leisure.keywords),
        };

        Self {
            defaults,
            gains,
            decay,
            dependency,
            thresholds,
            alcohol,
            leisure,
        }
    }
}

fn normalise_keywords(keywords: &[String]) -> Vec<String> {
    keywords
        .iter()
        .map(|keyword| keyword.trim().to_ascii_lowercase())
        .filter(|keyword| !keyword.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_falls_back_to_defaults() {
        let config = MotivationConfig::from(RawMotivationConfig::default());
        assert!(config.defaults.start <= config.defaults.max);
        assert!(config.defaults.start >= config.defaults.min);
        assert!(config.gains.task > 0.0);
        assert!(config
            .alcohol
            .trigger_keywords
            .contains(&"tavern".to_string()));
    }
}
