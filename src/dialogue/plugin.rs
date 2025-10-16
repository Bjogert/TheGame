//! Dialogue plugin wiring queue resources and systems.
use bevy::prelude::*;

use super::{
    broker::OpenAiDialogueBroker,
    errors::DialogueErrorKind,
    events::{DialogueRequestFailedEvent, DialogueResponseEvent},
    queue::{
        advance_dialogue_queue_timers, run_dialogue_request_queue, ActiveDialogueBroker,
        DialogueRateLimitConfig, DialogueRateLimitState, DialogueRequestQueue,
    },
    telemetry::{record_dialogue_telemetry, DialogueTelemetry},
};

const FALLBACK_DIALOGUE_TARGET: &str = "player";

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueRateLimitConfig>()
            .init_resource::<DialogueRateLimitState>()
            .init_resource::<DialogueRequestQueue>()
            .init_resource::<DialogueTelemetry>()
            .insert_resource(ActiveDialogueBroker::new(Box::new(
                OpenAiDialogueBroker::new(),
            )))
            .add_message::<DialogueResponseEvent>()
            .add_message::<DialogueRequestFailedEvent>()
            .add_systems(Startup, log_dialogue_provider)
            .add_systems(
                Update,
                (
                    advance_dialogue_queue_timers,
                    run_dialogue_request_queue,
                    record_dialogue_telemetry,
                    log_dialogue_events,
                )
                    .chain(),
            );
    }
}

fn log_dialogue_provider(broker: Res<ActiveDialogueBroker>) {
    info!(
        "DialoguePlugin initialised with provider: {}",
        broker.provider_kind()
    );
}

fn log_dialogue_events(
    mut responses: MessageReader<DialogueResponseEvent>,
    mut failures: MessageReader<DialogueRequestFailedEvent>,
) {
    for event in responses.read() {
        let response = &event.response;
        let target = response
            .target
            .map(|id| id.to_string())
            .unwrap_or_else(|| FALLBACK_DIALOGUE_TARGET.to_string());

        info!(
            "Dialogue response [{} | {} -> {} | {}]: {}",
            response.request_id.value(),
            response.speaker,
            target,
            response.provider,
            response.content
        );
    }

    for event in failures.read() {
        let error = &event.error;
        match &error.kind {
            DialogueErrorKind::RateLimited {
                retry_after_seconds,
            } => {
                warn!(
                    "Dialogue request {} rate limited for {:.2}s",
                    error.request_id.value(),
                    retry_after_seconds
                );
            }
            DialogueErrorKind::ProviderFailure { message } => {
                warn!(
                    "Dialogue provider failure ({}): {}",
                    error.provider, message
                );
            }
            DialogueErrorKind::ContextMissing { missing } => {
                warn!(
                    "Dialogue request {} missing context: {}",
                    error.request_id.value(),
                    missing
                );
            }
        }
    }
}
