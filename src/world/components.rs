//! Components used by the world module.
use bevy::prelude::*;

/// Marker component for the primary world camera, storing orientation state.
#[derive(Component)]
pub struct FlyCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub move_speed: f32,
    pub look_sensitivity: f32,
}

impl FlyCamera {
    pub fn new(yaw: f32, pitch: f32) -> Self {
        Self {
            yaw,
            pitch,
            move_speed: 10.0,
            look_sensitivity: 0.2,
        }
    }
}

/// Marker component identifying the main directional light (the "sun").
#[derive(Component, Default)]
pub struct PrimarySun;
