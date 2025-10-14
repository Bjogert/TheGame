//! Dialogue module hosting broker abstractions, request queueing, and context types.
pub mod broker;
pub mod errors;
pub mod events;
pub mod plugin;
pub mod queue;
pub mod types;

pub use broker::{DialogueBroker, DialogueProviderKind, LocalDialogueBroker};
pub use errors::{DialogueContextSource, DialogueError, DialogueErrorKind};
pub use events::{DialogueRequestFailedEvent, DialogueResponseEvent};
pub use plugin::DialoguePlugin;
pub use queue::{DialogueRateLimitConfig, DialogueRateLimitState, DialogueRequestQueue};
pub use types::{
    DialogueContext, DialogueContextEvent, DialogueRequest, DialogueRequestId, DialogueResponse,
    DialogueTopicHint, TradeContext, TradeContextReason, TradeDescriptor,
};
