// src/ui/speech_bubble/components.rs
//
// Speech bubble components for displaying NPC dialogue as screen-space UI.

use bevy::prelude::*;

use crate::npc::components::NpcId;

/// Marker component for speech bubble UI entities.
///
/// Speech bubbles are UI nodes positioned in screen space that track
/// the 3D world position of NPCs via camera projection.
#[derive(Component, Debug)]
pub struct SpeechBubble {
    /// The NPC ID this bubble is displaying dialogue for.
    npc_id: NpcId,

    /// The NPC entity this bubble tracks in 3D space.
    speaker_entity: Entity,

    /// The lifetime timer. When it expires, the bubble despawns.
    lifetime: Timer,
}

impl SpeechBubble {
    /// Create a new speech bubble tracking an NPC.
    pub fn new(npc_id: NpcId, speaker_entity: Entity, lifetime_secs: f32) -> Self {
        Self {
            npc_id,
            speaker_entity,
            lifetime: Timer::from_seconds(lifetime_secs, TimerMode::Once),
        }
    }

    /// Get the NPC ID this bubble belongs to.
    pub fn npc_id(&self) -> NpcId {
        self.npc_id
    }

    /// Get the speaker entity this bubble tracks.
    pub fn speaker(&self) -> Entity {
        self.speaker_entity
    }

    /// Tick the lifetime timer.
    pub fn tick(&mut self, delta: std::time::Duration) {
        self.lifetime.tick(delta);
    }

    /// Check if the bubble's lifetime has expired.
    pub fn is_finished(&self) -> bool {
        self.lifetime.is_finished()
    }

    /// Calculate the alpha fade value (1.0 = fully visible, 0.0 = transparent).
    ///
    /// Fades out during the final `fade_duration` seconds of lifetime.
    pub fn fade_alpha(&self, fade_duration: f32) -> f32 {
        let remaining = self.lifetime.remaining_secs();
        if remaining < fade_duration {
            remaining / fade_duration
        } else {
            1.0
        }
    }
}

/// Resource tracking active speech bubbles by NPC ID.
///
/// Ensures each NPC has at most one bubble at a time.
#[derive(Resource, Debug, Default)]
pub struct SpeechBubbleTracker {
    /// Maps NPC ID to the bubble entity currently displaying for that NPC.
    pub by_npc: std::collections::HashMap<NpcId, Entity>,
}

/// Resource containing settings for speech bubble behavior.
#[derive(Resource, Debug)]
pub struct SpeechBubbleSettings {
    /// How long bubbles remain visible (seconds).
    pub lifetime_seconds: f32,

    /// Duration of fade-out animation (seconds).
    pub fade_seconds: f32,

    /// Maximum distance from camera where bubbles are visible (world units).
    pub max_display_distance: f32,

    /// Vertical offset above NPC head in world space (world units).
    pub vertical_offset: f32,

    /// Font size for bubble text (points).
    pub font_size: f32,
}

impl Default for SpeechBubbleSettings {
    fn default() -> Self {
        Self {
            lifetime_seconds: 10.0,
            fade_seconds: 2.0,
            max_display_distance: 25.0,
            vertical_offset: 2.5,
            font_size: 15.0, // 25% smaller than original 20.0
        }
    }
}
