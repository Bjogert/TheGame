//! NPC-specific components and supporting resources.
use std::fmt;

use bevy::prelude::*;

use crate::dialogue::types::DialogueRequestId;

/// Unique identifier for an NPC.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct NpcId(u64);

impl NpcId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }
}

impl fmt::Display for NpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NPC-{:04}", self.0)
    }
}

/// Minimal identity data for debugging and future systems.
#[derive(Component, Debug, Clone)]
pub struct Identity {
    pub id: NpcId,
    pub display_name: String,
    pub age_years: f32,
}

impl Identity {
    pub fn new(id: NpcId, display_name: impl Into<String>, age_years: f32) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            age_years,
        }
    }
}

/// Describes a single scheduled activity starting at a fraction of the day.
#[derive(Debug, Clone)]
pub struct ScheduleEntry {
    pub start: f32,
    pub activity: String,
}

impl ScheduleEntry {
    pub fn new(start: f32, activity: impl Into<String>) -> Self {
        Self {
            start: start.rem_euclid(1.0),
            activity: activity.into(),
        }
    }
}

/// Daily schedule describing the activities an NPC performs.
#[derive(Component, Debug, Clone)]
pub struct DailySchedule {
    pub entries: Vec<ScheduleEntry>,
}

impl DailySchedule {
    pub fn new(mut entries: Vec<ScheduleEntry>) -> Self {
        entries.sort_by(|a, b| {
            a.start
                .partial_cmp(&b.start)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        Self { entries }
    }
}

/// Tracks the last activity assigned to an NPC (avoids spamming logs).
#[derive(Component, Debug, Default, Clone)]
pub struct ScheduleState {
    pub current_activity: String,
}

/// Controls how often schedules advance (seconds of simulation time).
#[derive(Resource)]
pub struct ScheduleTicker {
    pub interval_seconds: f32,
    accumulated: f32,
    pending_ticks: u32,
}

impl Default for ScheduleTicker {
    fn default() -> Self {
        Self {
            interval_seconds: 5.0,
            accumulated: 0.0,
            pending_ticks: 0,
        }
    }
}

impl ScheduleTicker {
    /// Accumulates delta time and returns how many ticks should fire.
    pub fn accumulate(&mut self, delta_seconds: f32) -> u32 {
        if self.interval_seconds <= f32::EPSILON {
            return 0;
        }

        self.accumulated += delta_seconds.max(0.0);
        let mut ticks = 0;
        while self.accumulated >= self.interval_seconds {
            self.accumulated -= self.interval_seconds;
            ticks += 1;
        }
        self.pending_ticks = self.pending_ticks.saturating_add(ticks);
        ticks
    }

    pub fn take_pending(&mut self) -> u32 {
        let ticks = self.pending_ticks;
        self.pending_ticks = 0;
        ticks
    }
}

/// Resource that issues monotonically increasing NPC ids.
#[derive(Resource, Default)]
pub struct NpcIdGenerator {
    next: u64,
}

impl NpcIdGenerator {
    pub fn next_id(&mut self) -> NpcId {
        let id = self.next;
        self.next += 1;
        NpcId::new(id)
    }
}

/// Simple locomotion controller tracking destinations and movement state.
#[derive(Component, Debug, Clone)]
pub struct NpcLocomotion {
    move_speed: f32,
    arrive_distance: f32,
    target: Option<MovementTarget>,
    state: LocomotionState,
    active_label: Option<String>,
}

impl NpcLocomotion {
    pub fn new(move_speed: f32, arrive_distance: f32) -> Self {
        Self {
            move_speed,
            arrive_distance,
            target: None,
            state: LocomotionState::Idle,
            active_label: None,
        }
    }

    pub fn move_speed(&self) -> f32 {
        self.move_speed
    }

    pub fn arrive_distance(&self) -> f32 {
        self.arrive_distance
    }

    pub fn target(&self) -> Option<MovementTarget> {
        self.target
    }

    pub fn state(&self) -> LocomotionState {
        self.state
    }

    pub fn active_label(&self) -> Option<&str> {
        self.active_label.as_deref()
    }

    /// Returns true when a new travel target is registered.
    pub fn set_target(&mut self, target: MovementTarget, label: impl Into<String>) -> bool {
        let label_string = label.into();
        let is_duplicate = self.state == LocomotionState::Moving
            && self.target == Some(target)
            && self
                .active_label
                .as_ref()
                .map(|existing| existing == &label_string)
                .unwrap_or(false);

        if is_duplicate {
            return false;
        }

        self.target = Some(target);
        self.state = LocomotionState::Moving;
        self.active_label = Some(label_string);
        true
    }

    pub fn clear_target(&mut self) {
        self.target = None;
        self.state = LocomotionState::Idle;
        self.active_label = None;
    }
}

impl Default for NpcLocomotion {
    fn default() -> Self {
        Self::new(2.5, 0.35)
    }
}

/// Where a locomotion controller should move.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MovementTarget {
    Entity(Entity),
}

/// Locomotion phase for logging and telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocomotionState {
    Idle,
    Moving,
}

/// Tracks when an NPC is engaged in a dialogue conversation.
#[derive(Component, Debug, Clone)]
pub struct InConversation {
    pub partner: NpcId,
    #[allow(dead_code)] // Will be used for Speaking state transitions in future
    pub request_id: DialogueRequestId,
    pub started_at: f32,
    pub state: ConversationState,
}

impl InConversation {
    pub fn new(
        partner: NpcId,
        request_id: DialogueRequestId,
        started_at: f32,
        state: ConversationState,
    ) -> Self {
        Self {
            partner,
            request_id,
            started_at,
            state,
        }
    }
}

/// State of an NPC conversation for coordinating movement and dialogue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationState {
    /// Walking toward conversation partner, API call in progress
    Approaching,
    /// Arrived at destination, waiting for API response
    WaitingAtDestination,
    /// Dialogue panel is visible, speaking
    #[allow(dead_code)] // Will be used when transitioning to speaking state
    Speaking,
}
