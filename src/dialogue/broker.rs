//! Dialogue broker trait and a local placeholder implementation.
use std::fmt;

use super::{
    errors::{DialogueContextSource, DialogueError, DialogueErrorKind},
    types::{
        DialogueContextEvent, DialogueRequest, DialogueRequestId, DialogueResponse,
        DialogueTopicHint, TradeContextReason,
    },
};

const EMPTY_PROMPT_ERROR: &str = "prompt cannot be empty";
const MANUAL_RETRY_PROMPT: &str = "retry later";
const MANUAL_RETRY_BACKOFF_SECONDS: f32 = 3.0;
const FALLBACK_TARGET_LABEL: &str = "player";
const SUMMARY_PREFIX: &str = "Summary:";
const SCHEDULE_NOTE_PREFIX: &str = "Schedule note:";
const CONTEXT_FALLBACK_MESSAGE: &str = "No notable context available.";

/// Dialogue provider flavours we can route to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueProviderKind {
    OpenAi,
}

impl fmt::Display for DialogueProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::OpenAi => "openai",
        };
        write!(f, "{}", label)
    }
}

/// Contract every dialogue backend must satisfy.
pub trait DialogueBroker: Send + Sync {
    fn provider_kind(&self) -> DialogueProviderKind;

    fn process(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> Result<DialogueResponse, DialogueError>;
}

/// Placeholder broker for the OpenAI backend until the API client lands.
pub struct OpenAiDialogueBroker;

impl OpenAiDialogueBroker {
    pub fn new() -> Self {
        Self
    }
}

impl DialogueBroker for OpenAiDialogueBroker {
    fn provider_kind(&self) -> DialogueProviderKind {
        DialogueProviderKind::OpenAi
    }

    fn process(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> Result<DialogueResponse, DialogueError> {
        if request.prompt.trim().is_empty() {
            return Err(DialogueError::new(
                request_id,
                self.provider_kind(),
                DialogueErrorKind::provider_failure(EMPTY_PROMPT_ERROR),
            ));
        }

        if request.prompt.eq_ignore_ascii_case(MANUAL_RETRY_PROMPT) {
            return Err(DialogueError::new(
                request_id,
                self.provider_kind(),
                DialogueErrorKind::rate_limited(MANUAL_RETRY_BACKOFF_SECONDS),
            ));
        }

        match request.topic_hint {
            DialogueTopicHint::Trade => {
                if request.context.summary.is_none() {
                    return Err(DialogueError::new(
                        request_id,
                        self.provider_kind(),
                        DialogueErrorKind::context_missing(DialogueContextSource::InventoryState),
                    ));
                }

                if !request
                    .context
                    .events
                    .iter()
                    .any(|event| matches!(event, DialogueContextEvent::Trade(_)))
                {
                    return Err(DialogueError::new(
                        request_id,
                        self.provider_kind(),
                        DialogueErrorKind::context_missing(DialogueContextSource::TradeHistory),
                    ));
                }
            }
            DialogueTopicHint::Schedule => {
                if !request
                    .context
                    .events
                    .iter()
                    .any(|event| matches!(event, DialogueContextEvent::ScheduleUpdate { .. }))
                {
                    return Err(DialogueError::new(
                        request_id,
                        self.provider_kind(),
                        DialogueErrorKind::context_missing(DialogueContextSource::ScheduleState),
                    ));
                }
            }
            DialogueTopicHint::Status => {}
        }

        let mut segments = Vec::new();
        segments.push(request.prompt.trim().to_string());

        if let Some(summary) = &request.context.summary {
            if !summary.trim().is_empty() {
                segments.push(format!("{} {}", SUMMARY_PREFIX, summary.trim()));
            }
        }

        let target_label = request
            .target
            .map(|id| id.to_string())
            .unwrap_or_else(|| FALLBACK_TARGET_LABEL.to_string());
        segments.push(format!("Target: {}", target_label));

        for event in &request.context.events {
            match event {
                DialogueContextEvent::Trade(trade) => {
                    let quantity = trade.descriptor.quantity;
                    let label = &trade.descriptor.label;
                    let action = match trade.reason {
                        TradeContextReason::Production => "produced",
                        TradeContextReason::Processing => "processed",
                        TradeContextReason::Exchange => "exchanged",
                    };
                    let mut detail = format!(
                        "On day {} they {} {} {}",
                        trade.day, action, quantity, label
                    );
                    if let Some(target) = trade.to {
                        detail.push_str(&format!(" with {}", target));
                    }
                    if let Some(source) = trade.from {
                        detail.push_str(&format!(" after receiving it from {}", source));
                    }
                    detail.push('.');
                    segments.push(detail);
                }
                DialogueContextEvent::ScheduleUpdate { description } => {
                    segments.push(format!("{} {}.", SCHEDULE_NOTE_PREFIX, description));
                }
            }
        }

        if segments.is_empty() {
            segments.push(CONTEXT_FALLBACK_MESSAGE.to_string());
        }

        let content = segments.join(" ");
        Ok(DialogueResponse::new(
            request_id,
            self.provider_kind(),
            request.speaker,
            request.target,
            content,
        ))
    }
}
