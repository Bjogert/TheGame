//! NPC-specific components and supporting resources.
use std::fmt;

use bevy::prelude::*;

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
#[allow(dead_code)]
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
        entries.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
        Self { entries }
    }
}

/// Tracks the last activity assigned to an NPC (avoids spamming logs).
#[derive(Component, Debug, Default, Clone)]
pub struct ScheduleState {
    pub current_activity: String,
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
