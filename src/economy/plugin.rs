//! Economy plugin wiring placeholder trade systems.
use bevy::prelude::*;

use crate::{npc::systems::spawn_debug_npcs, world::time::advance_world_clock};

use super::{
    events::TradeCompletedEvent,
    resources::MicroTradeLoopState,
    systems::{assign_placeholder_professions, process_micro_trade_loop},
};

pub struct EconomyPlugin;

impl Plugin for EconomyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MicroTradeLoopState>()
            .add_event::<TradeCompletedEvent>()
            .add_systems(
                Startup,
                assign_placeholder_professions.after(spawn_debug_npcs),
            )
            .add_systems(Update, process_micro_trade_loop.after(advance_world_clock));
    }
}
