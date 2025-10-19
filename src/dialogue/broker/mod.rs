//! Dialogue broker trait and OpenAI-backed implementation.

pub mod config;
pub mod openai;

use std::fmt;

use super::{
    errors::DialogueError,
    status::DialogueConnectionState,
    types::{DialogueRequest, DialogueRequestId, DialogueResponse},
};

pub use openai::OpenAiDialogueBroker;

/// Dialogue provider flavours we can route to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueProviderKind {
    OpenAi,
}

impl fmt::Display for DialogueProviderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::OpenAi => "OpenAi",
        };
        write!(f, "{}", label)
    }
}

/// Contract every dialogue backend must satisfy.
pub trait DialogueBroker: Send + Sync {
    fn provider_kind(&self) -> DialogueProviderKind;

    fn connection_state(&self) -> DialogueConnectionState;

    fn process(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> Result<DialogueResponse, DialogueError>;
}
