//! NPC plugin wiring identity data and debug spawners.
use bevy::prelude::*;

use crate::{
    npc::{
        components::{NpcIdGenerator, ScheduleTicker},
        systems::{drive_npc_locomotion, spawn_debug_npcs, tick_schedule_state},
    },
    world::systems::spawn_world_environment,
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<NpcIdGenerator>()
            .init_resource::<ScheduleTicker>()
            .add_systems(Startup, spawn_debug_npcs.after(spawn_world_environment))
            .add_systems(Update, (tick_schedule_state, drive_npc_locomotion));
    }
}
