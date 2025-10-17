//! Economy data loading and recipe registry.
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use bevy::{log::warn, prelude::Resource};
use serde::Deserialize;

use super::components::{Profession, TradeGood};

const ECONOMY_CONFIG_PATH: &str = "config/economy.toml";

#[derive(Debug, Clone, Deserialize)]
pub struct EconomyConfig {
    pub recipes: Vec<RecipeConfig>,
    #[serde(default)]
    pub daily_requests: Vec<DailyRequestConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecipeConfig {
    pub id: String,
    pub actor: Profession,
    #[serde(default)]
    pub produces: Vec<ProductConfig>,
    #[serde(default)]
    pub consumes: Vec<ProductConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProductConfig {
    pub good: TradeGood,
    pub quantity: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DailyRequestConfig {
    pub requester: Profession,
    pub good: TradeGood,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct Recipe {
    pub id: String,
    pub actor: Profession,
    pub produces: Vec<RecipeOutput>,
    pub consumes: Vec<RecipeInput>,
}

#[derive(Debug, Clone)]
pub struct RecipeInput {
    pub good: TradeGood,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct RecipeOutput {
    pub good: TradeGood,
    pub quantity: u32,
}

#[derive(Debug, Clone)]
pub struct DailyRequest {
    pub requester: Profession,
    pub good: TradeGood,
    pub quantity: u32,
}

#[derive(Resource, Debug, Clone)]
pub struct EconomyRegistry {
    recipes: HashMap<String, Recipe>,
    recipe_by_output: HashMap<TradeGood, String>,
    daily_requests: Vec<DailyRequest>,
}

impl EconomyRegistry {
    fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let data =
            fs::read_to_string(&path).map_err(|err| format!("unable to read file: {err}"))?;
        let config: EconomyConfig =
            toml::from_str(&data).map_err(|err| format!("invalid economy config: {err}"))?;
        Self::from_config(config)
    }

    fn from_config(config: EconomyConfig) -> Result<Self, String> {
        if config.recipes.is_empty() {
            return Err("economy config must define at least one recipe".to_string());
        }

        let mut recipes = HashMap::new();
        let mut recipe_by_output = HashMap::new();

        for recipe in config.recipes {
            if recipe.id.trim().is_empty() {
                return Err("recipe id cannot be empty".to_string());
            }

            if recipe.produces.is_empty() {
                return Err(format!(
                    "recipe '{}' must produce at least one good",
                    recipe.id
                ));
            }

            let converted = Recipe {
                id: recipe.id.clone(),
                actor: recipe.actor,
                produces: recipe
                    .produces
                    .into_iter()
                    .map(|product| RecipeOutput {
                        good: product.good,
                        quantity: product.quantity.max(1),
                    })
                    .collect(),
                consumes: recipe
                    .consumes
                    .into_iter()
                    .map(|product| RecipeInput {
                        good: product.good,
                        quantity: product.quantity.max(1),
                    })
                    .collect(),
            };

            for output in &converted.produces {
                if recipe_by_output.contains_key(&output.good) {
                    return Err(format!(
                        "multiple recipes produce the same good {:?}",
                        output.good
                    ));
                }
                recipe_by_output.insert(output.good, converted.id.clone());
            }

            recipes.insert(converted.id.clone(), converted);
        }

        let daily_requests = config
            .daily_requests
            .into_iter()
            .map(|request| DailyRequest {
                requester: request.requester,
                good: request.good,
                quantity: request.quantity.max(1),
            })
            .collect();

        Ok(Self {
            recipes,
            recipe_by_output,
            daily_requests,
        })
    }

    fn fallback() -> Self {
        let fallback_config = EconomyConfig {
            recipes: vec![
                RecipeConfig {
                    id: "grain_harvest".to_string(),
                    actor: Profession::Farmer,
                    produces: vec![ProductConfig {
                        good: TradeGood::Grain,
                        quantity: 1,
                    }],
                    consumes: vec![],
                },
                RecipeConfig {
                    id: "flour_milling".to_string(),
                    actor: Profession::Miller,
                    produces: vec![ProductConfig {
                        good: TradeGood::Flour,
                        quantity: 1,
                    }],
                    consumes: vec![ProductConfig {
                        good: TradeGood::Grain,
                        quantity: 1,
                    }],
                },
                RecipeConfig {
                    id: "toolsmithing".to_string(),
                    actor: Profession::Blacksmith,
                    produces: vec![ProductConfig {
                        good: TradeGood::Tools,
                        quantity: 1,
                    }],
                    consumes: vec![ProductConfig {
                        good: TradeGood::Flour,
                        quantity: 1,
                    }],
                },
            ],
            daily_requests: vec![DailyRequestConfig {
                requester: Profession::Farmer,
                good: TradeGood::Tools,
                quantity: 1,
            }],
        };

        Self::from_config(fallback_config).expect("fallback economy config should be valid")
    }

    pub fn recipe(&self, id: &str) -> Option<&Recipe> {
        self.recipes.get(id)
    }

    pub fn recipe_for_output(&self, good: TradeGood) -> Option<&Recipe> {
        self.recipe_by_output
            .get(&good)
            .and_then(|id| self.recipes.get(id))
    }

    pub fn daily_requests(&self) -> &[DailyRequest] {
        &self.daily_requests
    }
}

impl Default for EconomyRegistry {
    fn default() -> Self {
        match Self::load_from_file(ECONOMY_CONFIG_PATH) {
            Ok(registry) => registry,
            Err(error) => {
                warn!(
                    "Failed to load economy config from {}: {error}. Falling back to defaults.",
                    ECONOMY_CONFIG_PATH
                );
                Self::fallback()
            }
        }
    }
}
