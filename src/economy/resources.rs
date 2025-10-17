//! Economy resources for economy task execution and visuals.
use std::collections::HashMap;

use bevy::{
    ecs::world::FromWorld,
    math::primitives::Cuboid,
    prelude::{default, Assets, Color, Entity, Handle, Mesh, Resource, StandardMaterial, World},
};

use crate::economy::components::{Profession, TradeGood};

pub const PLACEHOLDER_SIZE: f32 = 0.32;

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

/// Shared mesh/material handles for placeholder goods.
#[derive(Resource, Debug)]
pub struct TradeGoodPlaceholderVisuals {
    mesh: Handle<Mesh>,
    materials: HashMap<TradeGood, Handle<StandardMaterial>>,
}

impl TradeGoodPlaceholderVisuals {
    pub fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }

    pub fn material(&self, good: TradeGood) -> Handle<StandardMaterial> {
        self.materials.get(&good).cloned().unwrap_or_else(|| {
            panic!("missing placeholder material for good {:?}", good);
        })
    }
}

impl FromWorld for TradeGoodPlaceholderVisuals {
    fn from_world(world: &mut World) -> Self {
        let mut meshes = world.resource_mut::<Assets<Mesh>>();
        let mesh = meshes.add(Mesh::from(Cuboid::new(
            PLACEHOLDER_SIZE,
            PLACEHOLDER_SIZE,
            PLACEHOLDER_SIZE,
        )));

        let mut materials_assets = world.resource_mut::<Assets<StandardMaterial>>();
        let mut materials = HashMap::new();

        let color_map = [
            (TradeGood::Grain, Color::srgb_u8(214, 181, 102)),
            (TradeGood::Flour, Color::srgb_u8(236, 235, 230)),
            (TradeGood::Tools, Color::srgb_u8(110, 118, 132)),
        ];

        for (good, color) in color_map {
            let handle = materials_assets.add(StandardMaterial {
                base_color: color,
                perceptual_roughness: 0.45,
                metallic: 0.05,
                ..default()
            });
            materials.insert(good, handle);
        }

        Self { mesh, materials }
    }
}
