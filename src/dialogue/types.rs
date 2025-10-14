//! Shared request/response types exposed by the dialogue module.
use crate::npc::components::NpcId;

/// Identifier assigned to queued dialogue requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct DialogueRequestId(u64);

impl DialogueRequestId {
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    pub fn value(self) -> u64 {
        self.0
    }
}

/// Hint to help providers frame responses without full prompt templates yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DialogueTopicHint {
    #[default]
    Status,
    Trade,
    Schedule,
}

/// Dialogue request describing who is speaking, the target, and prompt context.
#[derive(Debug, Clone)]
pub struct DialogueRequest {
    pub speaker: NpcId,
    pub target: Option<NpcId>,
    pub prompt: String,
    pub topic_hint: DialogueTopicHint,
    pub context: DialogueContext,
}

impl DialogueRequest {
    pub fn new(
        speaker: NpcId,
        target: Option<NpcId>,
        prompt: impl Into<String>,
        topic_hint: DialogueTopicHint,
        context: DialogueContext,
    ) -> Self {
        Self {
            speaker,
            target,
            prompt: prompt.into(),
            topic_hint,
            context,
        }
    }
}

/// Result returned by dialogue providers.
#[derive(Debug, Clone)]
pub struct DialogueResponse {
    pub request_id: DialogueRequestId,
    pub provider: DialogueProviderKind,
    pub speaker: NpcId,
    pub target: Option<NpcId>,
    pub content: String,
}

impl DialogueResponse {
    pub fn new(
        request_id: DialogueRequestId,
        provider: DialogueProviderKind,
        speaker: NpcId,
        target: Option<NpcId>,
        content: impl Into<String>,
    ) -> Self {
        Self {
            request_id,
            provider,
            speaker,
            target,
            content: content.into(),
        }
    }
}

/// High level context summary plus a list of structured events.
#[derive(Debug, Clone, Default)]
pub struct DialogueContext {
    pub summary: Option<String>,
    pub events: Vec<DialogueContextEvent>,
}

impl DialogueContext {
    pub fn with_events(events: Vec<DialogueContextEvent>) -> Self {
        Self {
            summary: None,
            events,
        }
    }
}

/// Context event categories provided to dialogue providers.
#[derive(Debug, Clone)]
pub enum DialogueContextEvent {
    Trade(TradeContext),
    ScheduleUpdate { description: String },
}

/// Trade-specific context that dialogue can reference.
#[derive(Debug, Clone)]
pub struct TradeContext {
    pub day: u64,
    pub from: Option<NpcId>,
    pub to: Option<NpcId>,
    pub descriptor: TradeDescriptor,
    pub reason: TradeContextReason,
}

/// Descriptor describing the traded good in simple language.
#[derive(Debug, Clone)]
pub struct TradeDescriptor {
    pub label: String,
    pub quantity: u32,
}

impl TradeDescriptor {
    pub fn new(label: impl Into<String>, quantity: u32) -> Self {
        Self {
            label: label.into(),
            quantity,
        }
    }
}

/// Why a trade occurred (production, processing, or exchange).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeContextReason {
    Production,
    Processing,
    Exchange,
}

// DialogueProviderKind is defined in broker.rs but referenced here.
use super::broker::DialogueProviderKind;
