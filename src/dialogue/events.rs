//! Events emitted by the dialogue queue runner.
use bevy::prelude::{Event, Message};

use super::{errors::DialogueError, types::DialogueResponse};

/// Fired when a dialogue request succeeds.
#[derive(Event, Message, Debug, Clone)]
pub struct DialogueResponseEvent {
    pub response: DialogueResponse,
}

/// Fired when a dialogue request fails after exhausting retries.
#[derive(Event, Message, Debug, Clone)]
pub struct DialogueRequestFailedEvent {
    pub error: DialogueError,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::{
        broker::DialogueProviderKind,
        errors::{DialogueError, DialogueErrorKind},
        types::{DialogueRequestId, DialogueResponse},
    };
    use crate::npc::components::NpcId;

    #[test]
    fn wraps_dialogue_payloads() {
        let speaker = NpcId::new(3);
        let request_id = DialogueRequestId::new(11);
        let response = DialogueResponse::new(
            request_id,
            DialogueProviderKind::OpenAi,
            speaker,
            None,
            "Hello there",
        );

        let response_event = DialogueResponseEvent { response };
        assert_eq!(response_event.response.content, "Hello there");
        assert_eq!(response_event.response.request_id.value(), 11);

        let error = DialogueError::new(
            request_id,
            DialogueProviderKind::OpenAi,
            DialogueErrorKind::provider_failure("boom"),
        );
        let failure_event = DialogueRequestFailedEvent { error };
        assert!(matches!(
            failure_event.error.kind,
            DialogueErrorKind::ProviderFailure { .. }
        ));
        assert_eq!(failure_event.error.request_id.value(), 11);
    }
}
