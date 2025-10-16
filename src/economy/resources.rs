//! Economy resources for placeholder trade loops.
use std::collections::HashMap;

use bevy::prelude::{Entity, Resource};

use crate::economy::components::Profession;

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
