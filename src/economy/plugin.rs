//! Economy plugin wiring placeholder trade systems.
use bevy::{ecs::schedule::IntoScheduleConfigs, prelude::*};

use crate::{
    npc::systems::spawn_debug_npcs,
    world::{systems::spawn_world_environment, time::advance_world_clock},
};

use super::{
    data::EconomyRegistry,
    dependency::EconomyDependencyMatrix,
    events::{ProfessionDependencyUpdateEvent, TradeCompletedEvent},
    resources::{
        ProfessionCrateRegistry, TradeGoodPlaceholderRegistry, TradeGoodPlaceholderVisuals,
    },
    systems::{
        advance_actor_tasks, assign_placeholder_professions, prepare_economy_day,
        spawn_profession_crates,
    },
    tasks::{ActorTaskQueues, EconomyDayState},
};

const SYSTEM_ACTOR_LABEL: &str = "system";

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EconomyRegistry>()
            .init_resource::<ProfessionCrateRegistry>()
            .init_resource::<TradeGoodPlaceholderRegistry>()
            .init_resource::<TradeGoodPlaceholderVisuals>()
            .init_resource::<ActorTaskQueues>()
            .init_resource::<EconomyDayState>()
            .init_resource::<EconomyDependencyMatrix>()
            .add_message::<TradeCompletedEvent>()
            .add_message::<ProfessionDependencyUpdateEvent>()
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
                (prepare_economy_day, advance_actor_tasks)
                    .chain()
                    .after(advance_world_clock),
            )
            .add_systems(Update, log_trade_events);
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
