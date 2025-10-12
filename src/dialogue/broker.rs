//! Dialogue broker abstractions and provider registry.
use std::{collections::HashMap, fmt, time::Duration};

use bevy::prelude::*;

use super::queue::DialogueRequest;

/// Supported dialogue providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueProvider {
    OpenAi,
    Anthropic,
    Local,
}

impl DialogueProvider {
    pub const fn as_str(&self) -> &'static str {
        match self {
            DialogueProvider::OpenAi => "openai",
            DialogueProvider::Anthropic => "anthropic",
            DialogueProvider::Local => "local",
        }
    }
}

impl fmt::Display for DialogueProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Unique identifier assigned to dialogue requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DialogueRequestId(u64);

impl DialogueRequestId {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for DialogueRequestId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "REQ-{:05}", self.0)
    }
}

/// Errors that can occur when dispatching dialogue requests.
#[derive(Debug, Clone)]
pub enum DialogueError {
    Timeout {
        request_id: DialogueRequestId,
        elapsed: Duration,
    },
    Throttled {
        request_id: DialogueRequestId,
        retry_after: Duration,
    },
    Transport {
        request_id: DialogueRequestId,
        message: String,
    },
    UnsupportedProvider {
        request_id: DialogueRequestId,
        provider: DialogueProvider,
    },
    Cancelled {
        request_id: DialogueRequestId,
        reason: String,
    },
}

impl DialogueError {
    pub fn request_id(&self) -> DialogueRequestId {
        match self {
            DialogueError::Timeout { request_id, .. }
            | DialogueError::Throttled { request_id, .. }
            | DialogueError::Transport { request_id, .. }
            | DialogueError::UnsupportedProvider { request_id, .. }
            | DialogueError::Cancelled { request_id, .. } => *request_id,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            DialogueError::Timeout { .. }
                | DialogueError::Throttled { .. }
                | DialogueError::Transport { .. }
        )
    }
}

/// Result of submitting a dialogue request to a provider.
#[derive(Debug, Clone)]
pub struct DialogueSubmission {
    pub request_id: DialogueRequestId,
    pub status: DialogueSubmissionStatus,
}

/// Provider acknowledgement for a dialogue request.
#[derive(Debug, Clone)]
pub enum DialogueSubmissionStatus {
    Accepted { estimated_latency: Duration },
    Deferred { retry_after: Duration },
}

/// Trait implemented by provider backends.
pub trait DialogueBroker: Send + Sync + 'static {
    fn provider(&self) -> DialogueProvider;

    fn submit(&self, request: &DialogueRequest) -> Result<DialogueSubmission, DialogueError>;
}

/// Resource tracking registered dialogue brokers.
#[derive(Resource, Default)]
pub struct DialogueBrokerRegistry {
    brokers: HashMap<DialogueProvider, Box<dyn DialogueBroker>>,
}

impl DialogueBrokerRegistry {
    pub fn register(&mut self, broker: Box<dyn DialogueBroker>) {
        let provider = broker.provider();
        if self.brokers.insert(provider, broker).is_some() {
            warn!(
                target: "dialogue",
                "Replaced existing broker for provider {provider}"
            );
        }
    }

    pub fn dispatch(&self, request: &DialogueRequest) -> Result<DialogueSubmission, DialogueError> {
        if let Some(broker) = self.brokers.get(&request.provider) {
            broker.submit(request)
        } else {
            Err(DialogueError::UnsupportedProvider {
                request_id: request.id,
                provider: request.provider,
            })
        }
    }

    pub fn is_registered(&self, provider: DialogueProvider) -> bool {
        self.brokers.contains_key(&provider)
    }
}

/// Minimal local broker stub used during development.
#[derive(Default)]
pub struct LocalEchoBroker;

impl DialogueBroker for LocalEchoBroker {
    fn provider(&self) -> DialogueProvider {
        DialogueProvider::Local
    }

    fn submit(&self, request: &DialogueRequest) -> Result<DialogueSubmission, DialogueError> {
        info!(
            target: "dialogue",
            "Dispatching {} via local echo broker (NPC: {:?})",
            request.id,
            request.npc_id
        );
        Ok(DialogueSubmission {
            request_id: request.id,
            status: DialogueSubmissionStatus::Accepted {
                estimated_latency: Duration::from_millis(50),
            },
        })
    }
}
