//! Economy plugin wiring placeholder trade systems.
use bevy::{ecs::schedule::IntoScheduleConfigs, prelude::*};

use crate::{
    npc::systems::spawn_debug_npcs,
    world::{systems::spawn_world_environment, time::advance_world_clock},
};

use super::{
    events::TradeCompletedEvent,
    resources::{MicroTradeLoopState, ProfessionCrateRegistry},
    systems::{assign_placeholder_professions, process_micro_trade_loop, spawn_profession_crates},
};

const SYSTEM_ACTOR_LABEL: &str = "system";

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MicroTradeLoopState>()
            .init_resource::<ProfessionCrateRegistry>()
            .add_message::<TradeCompletedEvent>()
            .add_systems(
                Startup,
                spawn_profession_crates.after(spawn_world_environment),
            )
            .add_systems(
                Startup,
                assign_placeholder_professions.after(spawn_debug_npcs),
            )
            .add_systems(
                Update,
                (
                    process_micro_trade_loop.after(advance_world_clock),
                    log_trade_events,
                )
                    .chain(),
            );
    }
}

fn log_trade_events(mut events: MessageReader<TradeCompletedEvent>) {
    for event in events.read() {
        let from = event
            .from
            .map(|id| id.to_string())
            .unwrap_or_else(|| SYSTEM_ACTOR_LABEL.to_string());
        let to = event
            .to
            .map(|id| id.to_string())
            .unwrap_or_else(|| SYSTEM_ACTOR_LABEL.to_string());

        info!(
            "Trade event day {}: {} -> {} | {} x{} ({:?})",
            event.day,
            from,
            to,
            event.good.label(),
            event.quantity,
            event.reason
        );
    }
}
