//! NPC plugin wiring identity data and debug spawners.
use bevy::prelude::*;

use crate::{
    npc::{
        components::NpcIdGenerator,
        systems::{spawn_debug_npcs, update_schedule_state},
    },
    world::{systems::spawn_world_environment, time::advance_world_clock},
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NpcIdGenerator>()
            .add_systems(Startup, spawn_debug_npcs.after(spawn_world_environment))
            .add_systems(Update, update_schedule_state.after(advance_world_clock));
    }
}
