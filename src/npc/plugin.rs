//! NPC plugin wiring identity data and debug spawners.
use bevy::prelude::*;

use crate::{
    npc::{
        components::{NpcIdGenerator, ScheduleTicker},
        events::NpcActivityChangedEvent,
        motivation::{
            decay_npc_motivation, evaluate_dependency_impacts, reward_from_dialogue_responses,
            reward_from_leisure, reward_from_trade_events, track_dependency_satisfaction,
            DailyDependencyTracker, MotivationConfig,
        },
        systems::{
            cleanup_conversations, drive_npc_locomotion, orient_conversing_npcs, spawn_debug_npcs,
            start_conversations, tick_schedule_state,
        },
    },
    world::systems::spawn_world_environment,
};

pub struct NpcPlugin;

impl Plugin for NpcPlugin {
    fn build(&self, app: &mut App) {
        let motivation_config = MotivationConfig::load_or_default();
        app.insert_resource(motivation_config)
            .init_resource::<NpcIdGenerator>()
            .init_resource::<ScheduleTicker>()
            .init_resource::<DailyDependencyTracker>()
            .add_message::<NpcActivityChangedEvent>()
            .add_systems(Startup, spawn_debug_npcs.after(spawn_world_environment))
            .add_systems(
                Update,
                (
                    start_conversations,
                    cleanup_conversations,
                    tick_schedule_state,
                    reward_from_leisure,
                    reward_from_trade_events,
                    reward_from_dialogue_responses,
                    track_dependency_satisfaction,
                    evaluate_dependency_impacts,
                    decay_npc_motivation,
                    drive_npc_locomotion,
                    orient_conversing_npcs,
                )
                    .chain(),
            );
    }
}
