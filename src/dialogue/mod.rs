//! Dialogue module coordinates queued LLM interactions.
pub mod broker;
pub mod plugin;
pub mod queue;
pub mod systems;

pub use broker::{
    DialogueBroker, DialogueBrokerRegistry, DialogueError, DialogueProvider, DialogueRequestId,
    DialogueSubmission, DialogueSubmissionStatus, LocalEchoBroker,
};
pub use plugin::DialoguePlugin;
pub use queue::{DialogueQueueConfig, DialogueQueueMetrics, DialogueRequest, DialogueRequestQueue};
