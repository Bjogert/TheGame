//! NPC-specific events broadcast between systems.
use bevy::prelude::{Event, Message};

use super::components::NpcId;

/// Fired when an NPC transitions to a new scheduled activity.
#[derive(Event, Message, Debug, Clone)]
pub struct NpcActivityChangedEvent {
    pub npc: NpcId,
    pub activity: String,
    pub time_of_day: f32,
}
