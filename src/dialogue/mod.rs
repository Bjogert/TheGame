//! Dialogue module hosting broker abstractions, request queueing, and context types.
pub mod broker;
pub mod errors;
pub mod events;
pub mod plugin;
pub mod queue;
pub mod status;
pub mod telemetry;
pub mod types;

pub use plugin::DialoguePlugin;

#[cfg(test)]
mod tests {
    use super::{
        broker::{DialogueBroker, DialogueProviderKind, OpenAiDialogueBroker},
        errors::{DialogueError, DialogueErrorKind},
        events::{DialogueRequestFailedEvent, DialogueResponseEvent},
        queue::{DialogueRateLimitConfig, DialogueRateLimitState, DialogueRequestQueue},
        types::{
            DialogueContext, DialogueContextEvent, DialogueRequest, DialogueTopicHint,
            TradeContext, TradeContextReason, TradeDescriptor,
        },
    };
    use crate::npc::components::NpcId;

    #[test]
    fn reexports_are_usable() {
        let mut queue = DialogueRequestQueue::default();
        let request = DialogueRequest::new(
            NpcId::new(1),
            None,
            "hello world",
            DialogueTopicHint::Status,
            DialogueContext::default(),
        );
        let request_id = queue.enqueue(request);
        assert_eq!(request_id.value(), 0);
        assert!(queue.front_ready());

        let broker = OpenAiDialogueBroker::new();
        let response = broker
            .process(
                request_id,
                &DialogueRequest::new(
                    NpcId::new(1),
                    Some(NpcId::new(2)),
                    "How's the market?",
                    DialogueTopicHint::Status,
                    DialogueContext::default(),
                ),
            )
            .expect("stub broker should fabricate a response");
        assert_eq!(response.provider, DialogueProviderKind::OpenAi);

        let error = DialogueError::new(
            request_id,
            DialogueProviderKind::OpenAi,
            DialogueErrorKind::provider_failure("not yet implemented"),
        );

        let _failure_event = DialogueRequestFailedEvent {
            error: error.clone(),
        };
        let _response_event = DialogueResponseEvent {
            response: response.clone(),
        };

        let mut limits = DialogueRateLimitState::default();
        limits.record_success(NpcId::new(1), &DialogueRateLimitConfig::default());
        assert!(!limits.can_process(NpcId::new(1)));

        let trade_descriptor = TradeDescriptor::new("grain", 5);
        let trade_context = TradeContext {
            day: 1,
            from: Some(NpcId::new(1)),
            to: Some(NpcId::new(2)),
            descriptor: trade_descriptor,
            reason: TradeContextReason::Exchange,
        };
        let context_event = DialogueContextEvent::Trade(trade_context);
        assert!(matches!(context_event, DialogueContextEvent::Trade(_)));
    }
}
