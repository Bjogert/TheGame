//! Events emitted by the dialogue queue runner.
use bevy::prelude::Event;

use super::{errors::DialogueError, types::DialogueResponse};

/// Fired when a dialogue request succeeds.
#[derive(Event, Debug, Clone)]
pub struct DialogueResponseEvent {
    pub response: DialogueResponse,
}

/// Fired when a dialogue request fails after exhausting retries.
#[derive(Event, Debug, Clone)]
pub struct DialogueRequestFailedEvent {
    pub error: DialogueError,
}
