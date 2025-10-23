// src/ui/speech_bubble/mod.rs
//
// Speech bubble module for displaying NPC dialogue in world space.
//
// This module implements Text2d bubbles that exist in 3D space above NPCs:
// - Spawns from DialogueResponseEvent
// - Positioned using Transform in world coordinates
// - Billboard rotation to always face camera
// - Fades out and despawns after a lifetime
// - Culls based on distance for performance
// - (Future) Adjusts size based on NPC mood/personality

pub mod components;
pub mod plugin;
pub mod systems;

// Re-export commonly used items
#[allow(unused_imports)] // Used by systems internally
pub use components::{SpeechBubbleSettings, SpeechBubbleTracker};
pub use plugin::SpeechBubblePlugin;
