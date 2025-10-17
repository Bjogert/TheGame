//! Telemetry storage for dialogue responses and failures.
use std::{
    collections::VecDeque,
    fs::{create_dir_all, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use bevy::{log::warn, prelude::*};
use serde::Serialize;

use super::{
    errors::{DialogueError, DialogueErrorKind},
    events::{DialogueRequestFailedEvent, DialogueResponseEvent},
    types::DialogueResponse,
};

const DEFAULT_DIALOGUE_TELEMETRY_LOG_PATH: &str = "logs/dialogue_history.jsonl";

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
    mut log: ResMut<DialogueTelemetryLog>,
) {
    let now = time.elapsed_secs_f64();

    for event in responses.read() {
        let record = DialogueTelemetryRecord {
            occurred_at_seconds: now,
            event: DialogueTelemetryEvent::Response(event.response.clone()),
        };
        log.push(&record);
        telemetry.push(record);
    }

    for event in failures.read() {
        let record = DialogueTelemetryRecord {
            occurred_at_seconds: now,
            event: DialogueTelemetryEvent::Failure(event.error.clone()),
        };
        log.push(&record);
        telemetry.push(record);
    }
}

/// Rolling log that writes dialogue telemetry to disk for offline inspection.
#[derive(Resource, Debug)]
pub struct DialogueTelemetryLog {
    output_path: PathBuf,
    pending: Vec<DialogueTelemetryRecord>,
}

impl DialogueTelemetryLog {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            output_path: path.into(),
            pending: Vec::new(),
        }
    }

    pub fn push(&mut self, record: &DialogueTelemetryRecord) {
        self.pending.push(record.clone());
    }

    fn ensure_directory(&self) -> std::io::Result<()> {
        if let Some(parent) = self.output_path.parent() {
            create_dir_all(parent)?;
        }
        Ok(())
    }

    fn drain_pending(&mut self) -> Vec<DialogueTelemetryRecord> {
        std::mem::take(&mut self.pending)
    }

    pub fn flush(&mut self) -> std::io::Result<()> {
        if self.pending.is_empty() {
            return Ok(());
        }

        self.ensure_directory()?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output_path)?;

        for record in self.drain_pending() {
            let serialisable: SerializableDialogueTelemetryRecord = record.into();
            serde_json::to_writer(&mut file, &serialisable)?;
            file.write_all(b"\n")?;
        }

        file.flush()?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn path(&self) -> &Path {
        &self.output_path
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}

impl Default for DialogueTelemetryLog {
    fn default() -> Self {
        Self::new(DEFAULT_DIALOGUE_TELEMETRY_LOG_PATH)
    }
}

/// Flushes pending telemetry log entries to disk, logging a warning if persistence fails.
pub fn flush_dialogue_telemetry_log(mut log: ResMut<DialogueTelemetryLog>) {
    if let Err(err) = log.flush() {
        warn!(
            "Failed to persist dialogue telemetry to {:?}: {}",
            log.path(),
            err
        );
    }
}

#[derive(Serialize)]
struct SerializableDialogueTelemetryRecord {
    occurred_at_seconds: f64,
    event: SerializableDialogueTelemetryEvent,
}

impl From<DialogueTelemetryRecord> for SerializableDialogueTelemetryRecord {
    fn from(value: DialogueTelemetryRecord) -> Self {
        Self {
            occurred_at_seconds: value.occurred_at_seconds,
            event: value.event.into(),
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
enum SerializableDialogueTelemetryEvent {
    Response {
        request_id: u64,
        provider: String,
        speaker: String,
        target: Option<String>,
        content: String,
    },
    Failure {
        request_id: u64,
        provider: String,
        error: SerializableDialogueError,
    },
}

impl From<DialogueTelemetryEvent> for SerializableDialogueTelemetryEvent {
    fn from(value: DialogueTelemetryEvent) -> Self {
        match value {
            DialogueTelemetryEvent::Response(response) => Self::Response {
                request_id: response.request_id.value(),
                provider: response.provider.to_string(),
                speaker: response.speaker.to_string(),
                target: response.target.map(|id| id.to_string()),
                content: response.content,
            },
            DialogueTelemetryEvent::Failure(error) => Self::Failure {
                request_id: error.request_id.value(),
                provider: error.provider.to_string(),
                error: error.kind.into(),
            },
        }
    }
}

#[derive(Serialize)]
#[serde(tag = "error_kind", rename_all = "snake_case")]
enum SerializableDialogueError {
    RateLimited { retry_after_seconds: f32 },
    ProviderFailure { message: String },
    ContextMissing { missing: String },
}

impl From<DialogueErrorKind> for SerializableDialogueError {
    fn from(value: DialogueErrorKind) -> Self {
        match value {
            DialogueErrorKind::RateLimited {
                retry_after_seconds,
            } => Self::RateLimited {
                retry_after_seconds,
            },
            DialogueErrorKind::ProviderFailure { message } => Self::ProviderFailure { message },
            DialogueErrorKind::ContextMissing { missing } => Self::ContextMissing {
                missing: missing.to_string(),
            },
        }
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
    use serde_json::Value;
    use std::{env, fs, time::SystemTime};

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

    #[test]
    fn telemetry_log_writes_json_lines() {
        let temp_dir = env::temp_dir();
        let unique_suffix = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = temp_dir.join(format!("dialogue_log_test_{}.jsonl", unique_suffix));
        if path.exists() {
            let _ = fs::remove_file(&path);
        }

        let mut log = DialogueTelemetryLog::new(&path);

        let response_record = DialogueTelemetryRecord {
            occurred_at_seconds: 12.5,
            event: DialogueTelemetryEvent::Response(DialogueResponse::new(
                DialogueRequestId::new(9),
                DialogueProviderKind::OpenAi,
                NpcId::new(42),
                Some(NpcId::new(7)),
                "Greetings",
            )),
        };

        log.push(&response_record);
        log.flush().expect("telemetry log should flush");

        let raw = fs::read_to_string(&path).expect("log file should exist");
        let lines: Vec<_> = raw.lines().collect();
        assert_eq!(lines.len(), 1);

        let value: Value = serde_json::from_str(lines[0]).expect("json line should parse");
        assert_eq!(value["event"]["event_type"], "response");
        assert_eq!(value["event"]["provider"], "openai");
        assert_eq!(value["event"]["speaker"], "NPC-0042");
        assert_eq!(value["event"]["target"], "NPC-0007");

        let _ = fs::remove_file(&path);
    }
}
