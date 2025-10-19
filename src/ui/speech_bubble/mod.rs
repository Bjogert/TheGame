// src/ui/speech_bubble/mod.rs
//
// Speech bubble UI module for displaying NPC dialogue as screen-space UI.
//
// This module implements UI bubbles that track NPCs in 3D space:
// - Spawns from DialogueResponseEvent
// - Positions using camera.world_to_viewport() projection
// - Fades out and despawns after a lifetime
// - Culls based on distance for performance
// - (Future) Adjusts size based on NPC mood/personality

pub mod components;
pub mod plugin;
pub mod systems;

// Re-export commonly used items
#[allow(unused_imports)] // Used by systems internally
pub use components::{SpeechBubbleSettings, SpeechBubbleTracker, SpeechBubbleUiRoot};
pub use plugin::SpeechBubblePlugin;
