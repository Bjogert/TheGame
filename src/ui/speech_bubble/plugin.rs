// src/ui/speech_bubble/plugin.rs
//
// Plugin registration for speech bubble systems.

use bevy::prelude::*;

use super::components::{SpeechBubbleSettings, SpeechBubbleTracker};
use super::systems::{setup_speech_bubble_root, spawn_speech_bubbles, update_speech_bubbles};

/// Plugin providing speech bubble UI for NPC dialogue.
///
/// Spawns UI nodes positioned in screen space that track 3D NPCs using
/// camera projection (`world_to_viewport`). Bubbles fade out over time
/// and are automatically culled based on distance.
///
/// # System Ordering
///
/// 1. `setup_speech_bubble_root` - Creates the UI root overlay (runs in Startup)
/// 2. `spawn_speech_bubbles` - Listens to DialogueResponseEvent (runs in Update)
/// 3. `update_speech_bubbles` - Positions bubbles via world_to_viewport, handles lifetime/fade
///
/// # Dependencies
///
/// - `DialoguePlugin` must be registered before this plugin (provides DialogueResponseEvent)
/// - `NpcPlugin` must be registered (provides NPC entities and Identity components)
/// - `WorldPlugin` must be registered (provides FlyCamera for camera queries)
pub struct SpeechBubblePlugin;

impl Plugin for SpeechBubblePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SpeechBubbleSettings>()
            .init_resource::<SpeechBubbleTracker>()
            .add_systems(Startup, setup_speech_bubble_root)
            .add_systems(
                Update,
                (
                    spawn_speech_bubbles,
                    update_speech_bubbles.after(spawn_speech_bubbles),
                ),
            );

        info!("SpeechBubblePlugin registered");
    }
}
