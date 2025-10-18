use bevy::{math::primitives::Cuboid, prelude::*};

use crate::npc::components::Identity;

use super::super::{
    components::{Inventory, Profession, ProfessionCrate},
    resources::ProfessionCrateRegistry,
};

pub(super) const FARMER_NAME: &str = "Alric";
pub(super) const MILLER_NAME: &str = "Bryn";
pub(super) const BLACKSMITH_NAME: &str = "Cedric";

const CRATE_MESH_DIMENSIONS: (f32, f32, f32) = (0.9, 0.6, 0.9);
const CRATE_PERCEPTUAL_ROUGHNESS: f32 = 0.6;
const CRATE_METALLIC: f32 = 0.1;
const CRATE_HEIGHT: f32 = 0.25;

#[derive(Clone, Copy)]
struct ProfessionCrateSpec {
    profession: Profession,
    translation: Vec3,
    color: (u8, u8, u8),
}

const PROFESSION_CRATE_SPECS: [ProfessionCrateSpec; 3] = [
    ProfessionCrateSpec {
        profession: Profession::Farmer,
        translation: Vec3::new(8.0, CRATE_HEIGHT, 3.0),
        color: (190, 150, 80),
    },
    ProfessionCrateSpec {
        profession: Profession::Miller,
        translation: Vec3::new(0.0, CRATE_HEIGHT, -6.5),
        color: (140, 170, 215),
    },
    ProfessionCrateSpec {
        profession: Profession::Blacksmith,
        translation: Vec3::new(-6.0, CRATE_HEIGHT, 1.5),
        color: (110, 110, 130),
    },
];

/// Spawns placeholder crate entities representing profession work spots.
pub fn spawn_profession_crates(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut registry: ResMut<ProfessionCrateRegistry>,
) {
    for spec in PROFESSION_CRATE_SPECS {
        if registry.get(spec.profession).is_some() {
            continue;
        }

        let color = Color::srgb_u8(spec.color.0, spec.color.1, spec.color.2);
        let entity = commands
            .spawn((
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(
                    CRATE_MESH_DIMENSIONS.0,
                    CRATE_MESH_DIMENSIONS.1,
                    CRATE_MESH_DIMENSIONS.2,
                )))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: CRATE_PERCEPTUAL_ROUGHNESS,
                    metallic: CRATE_METALLIC,
                    ..default()
                })),
                Transform::from_translation(spec.translation),
                ProfessionCrate {
                    profession: spec.profession,
                },
                Name::new(format!("{} crate", spec.profession.label())),
            ))
            .id();

        registry.insert(spec.profession, entity);
        info!(
            "Spawned {} crate at ({:.1}, {:.1}, {:.1})",
            spec.profession.label(),
            spec.translation.x,
            spec.translation.y,
            spec.translation.z
        );
    }
}

/// Assigns placeholder professions and empty inventories to debug NPCs.
pub fn assign_placeholder_professions(
    mut commands: Commands,
    query: Query<(Entity, &Identity), Without<Profession>>,
) {
    for (entity, identity) in query.iter() {
        let profession = match identity.display_name.as_str() {
            FARMER_NAME => Some(Profession::Farmer),
            MILLER_NAME => Some(Profession::Miller),
            BLACKSMITH_NAME => Some(Profession::Blacksmith),
            _ => None,
        };

        if let Some(profession) = profession {
            info!(
                "Assigning {} (age {:.1}) as {}",
                identity.display_name,
                identity.age_years,
                profession.label()
            );
            commands
                .entity(entity)
                .insert((profession, Inventory::default()));
        }
    }
}
