// src/ui/mod.rs
//
// UI module providing in-world and screen-space UI elements.
//
// Current features:
// - Speech bubbles (floating dialogue text above NPCs)
//
// Future features:
// - HUD overlays (health, resources, time-of-day)
// - Menus (pause, settings, save/load)
// - NPC info panels (hover tooltips, relationship status)

pub mod speech_bubble;

// Re-export the main plugin
pub use speech_bubble::SpeechBubblePlugin;
