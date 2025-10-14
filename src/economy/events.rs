//! Economy-specific events used by the micro trade loop.
use bevy::prelude::Event;

use crate::npc::components::NpcId;

use super::components::TradeGood;

#[derive(Event, Debug, Clone)]
pub struct TradeCompletedEvent {
    pub day: u64,
    pub from: Option<NpcId>,
    pub to: Option<NpcId>,
    pub good: TradeGood,
    pub quantity: u32,
    pub reason: TradeReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeReason {
    Production,
    Processing,
    Exchange,
}
