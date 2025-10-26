// src/ui/dialogue_panel/components.rs
//
// Components and resources for dialogue panel system.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::npc::components::NpcId;

/// Component attached to dialogue panel UI entities.
///
/// Tracks the NPC speaking, dialogue content, and lifetime timer.
#[derive(Component, Debug)]
pub struct DialoguePanel {
    /// The NPC ID this panel is displaying dialogue for.
    npc_id: NpcId,

    /// The display name of the speaking NPC.
    #[allow(dead_code)]
    speaker_name: String,

    /// The dialogue content being displayed.
    #[allow(dead_code)]
    content: String,

    /// The lifetime timer. When it expires, the panel despawns.
    lifetime: Timer,

    /// Duration of fade-out effect (stored for fade calculation).
    fade_duration: f32,
}

impl DialoguePanel {
    /// Create a new dialogue panel for an NPC.
    pub fn new(
        npc_id: NpcId,
        speaker_name: String,
        content: String,
        lifetime_secs: f32,
        fade_duration: f32,
    ) -> Self {
        Self {
            npc_id,
            speaker_name,
            content,
            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
            fade_duration,
        }
    }

    /// Get the NPC ID this panel belongs to.
    pub fn npc_id(&self) -> NpcId {
        self.npc_id
    }

    /// Tick the lifetime timer.
    pub fn tick(&mut self, delta: std::time::Duration) {
        self.lifetime.tick(delta);
    }

    /// Check if the panel's lifetime has expired.
    pub fn is_finished(&self) -> bool {
        self.lifetime.is_finished()
    }

    /// Calculate the alpha fade value (1.0 = fully visible, 0.0 = transparent).
    ///
    /// Fades out during the final `fade_duration` seconds of lifetime.
    pub fn fade_alpha(&self) -> f32 {
        let remaining = self.lifetime.remaining_secs();
        if remaining < self.fade_duration {
            remaining / self.fade_duration
        } else {
            1.0
        }
    }
}

/// Resource tracking the currently active dialogue panel.
///
/// Ensures only one panel is displayed at a time.
#[derive(Resource, Debug, Default)]
pub struct DialoguePanelTracker {
    /// The currently active panel entity, if any.
    pub active_panel: Option<Entity>,

    /// Maps NPC ID to their most recent dialogue (for reference).
    pub by_npc: HashMap<NpcId, Entity>,
}

/// Resource containing settings for dialogue panel behavior.
#[derive(Resource, Debug)]
pub struct DialoguePanelSettings {
    /// How long panels remain visible (seconds).
    pub lifetime_seconds: f32,

    /// Duration of fade-out animation (seconds).
    pub fade_seconds: f32,

    /// Panel width (pixels).
    pub panel_width: f32,

    /// Maximum panel height (pixels).
    pub panel_max_height: f32,

    /// Padding inside panel (pixels).
    pub padding: f32,

    /// Border width (pixels).
    pub border_width: f32,

    /// Offset from bottom edge of screen (pixels).
    pub bottom_offset: f32,

    /// Offset from right edge of screen (pixels).
    pub right_offset: f32,

    /// Font size for NPC name (points).
    pub name_font_size: f32,

    /// Font size for dialogue text (points).
    pub text_font_size: f32,

    /// Font size for icon emoji (points).
    pub icon_font_size: f32,
}

impl Default for DialoguePanelSettings {
    fn default() -> Self {
        Self {
            lifetime_seconds: 10.0,
            fade_seconds: 2.0,
            panel_width: 350.0,
            panel_max_height: 200.0,
            padding: 12.0,
            border_width: 2.0,
            bottom_offset: 20.0,
            right_offset: 20.0,
            name_font_size: 18.0,
            text_font_size: 16.0,
            icon_font_size: 20.0,
        }
    }
}
