//! Dialogue broker trait and a local placeholder implementation.
use std::fmt;

use super::{
    errors::{DialogueContextSource, DialogueError, DialogueErrorKind},
    types::{
        DialogueContextEvent, DialogueRequest, DialogueRequestId, DialogueResponse,
        DialogueTopicHint, TradeContextReason,
    },
};

/// Dialogue provider flavours we can route to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueProviderKind {
    OpenAi,
    Anthropic,
    LocalEcho,
}

impl fmt::Display for DialogueProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::OpenAi => "openai",
            Self::Anthropic => "anthropic",
            Self::LocalEcho => "local_echo",
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

/// Simple broker that fabricates responses locally for prototyping.
#[derive(Default)]
pub struct LocalDialogueBroker;

impl LocalDialogueBroker {
    pub fn new() -> Self {
        Self
    }
}

impl DialogueBroker for LocalDialogueBroker {
    fn provider_kind(&self) -> DialogueProviderKind {
        DialogueProviderKind::LocalEcho
    }

    fn process(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> Result<DialogueResponse, DialogueError> {
        if matches!(request.topic_hint, DialogueTopicHint::Trade)
            && !request
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

        let mut segments = Vec::new();
        segments.push(request.prompt.trim().to_string());

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
                    let mut detail = format!("They {} {} {}", action, quantity, label);
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
                    segments.push(format!("Schedule note: {}.", description));
                }
            }
        }

        if segments.is_empty() {
            segments.push("No notable context available.".to_string());
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
