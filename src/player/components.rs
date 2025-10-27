//! Components and resources for player interaction system.
use bevy::prelude::*;

use crate::npc::components::NpcId;

/// Marker component identifying the player entity (attached to camera).
#[derive(Component, Debug)]
pub struct Player;

/// Resource tracking player interaction state with nearby NPCs.
#[derive(Resource, Default, Debug)]
pub struct PlayerInteractionState {
    /// Information about the NPC the player is currently near and can interact with.
    pub nearby_npc: Option<NearbyNpcInfo>,
    /// Current NPC the player is conversing with (if any).
    pub active_dialogue: Option<NpcId>,
    /// Display name for the active NPC (cached for prompt building).
    pub active_npc_name: Option<String>,
    /// Last line spoken by the NPC.
    pub last_npc_line: Option<String>,
    /// Active response window entity (if shown).
    pub response_window: Option<Entity>,
}

/// Information about an NPC that is near the player.
#[derive(Debug, Clone)]
pub struct NearbyNpcInfo {
    /// Unique identifier for the NPC
    pub npc_id: NpcId,
    /// Display name of the NPC
    pub name: String,
    /// Distance from player to NPC (in world units)
    pub distance: f32,
}

/// Marker component for the player response UI window.
#[derive(Component, Debug)]
pub struct PlayerResponseWindow;

/// Component attached to each response button, carrying metadata used when clicked.
#[derive(Component, Debug)]
pub struct PlayerResponseButton {
    pub npc_id: NpcId,
    pub response_index: usize,
}
