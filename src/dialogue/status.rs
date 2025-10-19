//! Dialogue broker status tracking for runtime telemetry and UI.
use bevy::prelude::Resource;
use serde::Serialize;

use super::broker::DialogueProviderKind;

/// Connection state for the active dialogue broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DialogueConnectionState {
    Live,
    Fallback,
}

impl DialogueConnectionState {
    /// Human-readable label for logging.
    pub fn label(self) -> &'static str {
        match self {
            Self::Live => "live",
            Self::Fallback => "fallback",
        }
    }
}

/// Shared resource describing the currently active dialogue broker.
#[derive(Resource, Debug, Clone)]
pub struct DialogueBrokerStatus {
    provider: DialogueProviderKind,
    connection_state: DialogueConnectionState,
}

impl DialogueBrokerStatus {
    pub fn new(provider: DialogueProviderKind, connection_state: DialogueConnectionState) -> Self {
        Self {
            provider,
            connection_state,
        }
    }

    pub fn provider(&self) -> DialogueProviderKind {
        self.provider
    }

    pub fn connection_state(&self) -> DialogueConnectionState {
        self.connection_state
    }

    pub fn connection_label(&self) -> &'static str {
        self.connection_state.label()
    }

    pub fn to_snapshot(&self) -> DialogueBrokerStatusSnapshot {
        DialogueBrokerStatusSnapshot {
            provider: self.provider.to_string(),
            connection_state: self.connection_state,
        }
    }
}

/// Serializable snapshot for telemetry logging.
#[derive(Debug, Clone, Serialize)]
pub struct DialogueBrokerStatusSnapshot {
    pub provider: String,
    pub connection_state: DialogueConnectionState,
}
