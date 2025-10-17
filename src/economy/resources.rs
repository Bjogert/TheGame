//! Economy resources for placeholder trade loops.
use std::collections::HashMap;

use bevy::prelude::{Entity, Resource};

use crate::economy::components::{Profession, TradeGood};

#[derive(Resource, Debug, Default)]
pub struct MicroTradeLoopState {
    pub last_processed_day: Option<u64>,
}

/// Tracks the spawned crate entity for each profession.
#[derive(Resource, Debug, Default)]
pub struct ProfessionCrateRegistry {
    entries: HashMap<Profession, Entity>,
}

impl ProfessionCrateRegistry {
    pub fn insert(&mut self, profession: Profession, entity: Entity) {
        self.entries.insert(profession, entity);
    }

    pub fn get(&self, profession: Profession) -> Option<Entity> {
        self.entries.get(&profession).copied()
    }
}

/// Tracks placeholder entities spawned to represent goods near profession crates.
#[derive(Resource, Debug, Default)]
pub struct TradeGoodPlaceholderRegistry {
    entries: HashMap<(Profession, TradeGood), Entity>,
}

impl TradeGoodPlaceholderRegistry {
    pub fn contains(&self, profession: Profession, good: TradeGood) -> bool {
        self.entries.contains_key(&(profession, good))
    }

    pub fn insert(&mut self, profession: Profession, good: TradeGood, entity: Entity) {
        self.entries.insert((profession, good), entity);
    }

    pub fn take(&mut self, profession: Profession, good: TradeGood) -> Option<Entity> {
        self.entries.remove(&(profession, good))
    }
}
