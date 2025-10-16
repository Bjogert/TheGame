//! Telemetry storage for dialogue responses and failures.
use std::collections::VecDeque;

use bevy::prelude::*;

use super::{
    errors::DialogueError,
    events::{DialogueRequestFailedEvent, DialogueResponseEvent},
    types::DialogueResponse,
};

const DEFAULT_DIALOGUE_TELEMETRY_CAPACITY: usize = 64;

/// Rolling log of dialogue responses/failures for UI consumers.
#[derive(Resource, Debug)]
pub struct DialogueTelemetry {
    capacity: usize,
    records: VecDeque<DialogueTelemetryRecord>,
}

impl DialogueTelemetry {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
            records: VecDeque::new(),
        }
    }

    pub fn push(&mut self, record: DialogueTelemetryRecord) {
        while self.records.len() >= self.capacity {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }

    #[allow(dead_code)]
    pub fn records(&self) -> impl Iterator<Item = &DialogueTelemetryRecord> {
        self.records.iter()
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.records.len()
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for DialogueTelemetry {
    fn default() -> Self {
        Self::new(DEFAULT_DIALOGUE_TELEMETRY_CAPACITY)
    }
}

/// Single telemetry entry.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct DialogueTelemetryRecord {
    pub occurred_at_seconds: f64,
    pub event: DialogueTelemetryEvent,
}

/// Either a response or a failure.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum DialogueTelemetryEvent {
    Response(DialogueResponse),
    Failure(DialogueError),
}

/// System that records dialogue telemetry for later UI display.
pub fn record_dialogue_telemetry(
    time: Res<Time>,
    mut telemetry: ResMut<DialogueTelemetry>,
    mut responses: MessageReader<DialogueResponseEvent>,
    mut failures: MessageReader<DialogueRequestFailedEvent>,
) {
    let now = time.elapsed_secs_f64();

    for event in responses.read() {
        telemetry.push(DialogueTelemetryRecord {
            occurred_at_seconds: now,
            event: DialogueTelemetryEvent::Response(event.response.clone()),
        });
    }

    for event in failures.read() {
        telemetry.push(DialogueTelemetryRecord {
            occurred_at_seconds: now,
            event: DialogueTelemetryEvent::Failure(event.error.clone()),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::{
        broker::DialogueProviderKind,
        errors::DialogueErrorKind,
        types::{DialogueRequestId, DialogueResponse},
    };
    use crate::npc::components::NpcId;

    #[test]
    fn telemetry_drops_old_records_when_full() {
        let mut telemetry = DialogueTelemetry::new(2);
        telemetry.push(DialogueTelemetryRecord {
            occurred_at_seconds: 1.0,
            event: DialogueTelemetryEvent::Response(DialogueResponse::new(
                DialogueRequestId::new(1),
                DialogueProviderKind::OpenAi,
                NpcId::new(1),
                None,
                "Hello",
            )),
        });
        telemetry.push(DialogueTelemetryRecord {
            occurred_at_seconds: 2.0,
            event: DialogueTelemetryEvent::Failure(crate::dialogue::errors::DialogueError::new(
                DialogueRequestId::new(2),
                DialogueProviderKind::OpenAi,
                DialogueErrorKind::provider_failure("boom"),
            )),
        });
        telemetry.push(DialogueTelemetryRecord {
            occurred_at_seconds: 3.0,
            event: DialogueTelemetryEvent::Response(DialogueResponse::new(
                DialogueRequestId::new(3),
                DialogueProviderKind::OpenAi,
                NpcId::new(2),
                None,
                "Hi",
            )),
        });

        assert_eq!(telemetry.len(), 2);
        assert!(telemetry
            .records()
            .all(|record| record.occurred_at_seconds >= 2.0));
    }
}
