// src/ui/mod.rs
//
// UI module providing screen-space UI elements for HUD and dialogue.
//
// Current features:
// - Dialogue panels (bottom-right corner NPC dialogue display)
//
// Future features:
// - HUD overlays (health, resources, time-of-day)
// - Menus (pause, settings, save/load)
// - NPC info panels (hover tooltips, relationship status)

pub mod dialogue_panel;

// Re-export the main plugin
pub use dialogue_panel::UiPlugin;
