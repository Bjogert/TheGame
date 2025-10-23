// src/ui/speech_bubble/plugin.rs
//
// Plugin registration for speech bubble systems.

use bevy::prelude::*;

use super::components::{SpeechBubbleSettings, SpeechBubbleTracker};
use super::systems::{spawn_speech_bubbles, update_speech_bubbles};

/// Plugin providing speech bubble display for NPC dialogue.
///
/// Spawns Text2d entities in world space above NPCs with billboard rotation.
/// Bubbles fade out over time and are automatically culled based on distance.
///
/// # System Ordering
///
/// 1. `spawn_speech_bubbles` - Listens to DialogueResponseEvent (runs in Update)
/// 2. `update_speech_bubbles` - Updates Transform position, billboard rotation, handles lifetime/fade
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
