//! Dialogue plugin wiring queue resources, systems, instrumentation, and debug tooling.
use bevy::prelude::*;

use super::{
    broker::{DialogueBroker, OpenAiDialogueBroker},
    errors::DialogueErrorKind,
    events::{DialogueRequestFailedEvent, DialogueRequestedEvent, DialogueResponseEvent},
    queue::{
        advance_dialogue_queue_timers, poll_dialogue_tasks, run_dialogue_request_queue,
        ActiveDialogueBroker, DialogueRateLimitConfig, DialogueRateLimitState,
        DialogueRequestQueue, PendingDialogueTasks,
    },
    status::{DialogueBrokerStatus, DialogueConnectionState},
    telemetry::{
        flush_dialogue_telemetry_log, record_dialogue_telemetry, DialogueTelemetry,
        DialogueTelemetryEvent, DialogueTelemetryLog, DialogueTelemetryRecord,
    },
    types::{DialogueContext, DialogueRequest, DialogueTopicHint},
};
use crate::npc::components::Identity;

const FALLBACK_DIALOGUE_TARGET: &str = "player";
const DEBUG_DIALOGUE_PROBE_KEY: KeyCode = KeyCode::F7;
const DEBUG_DIALOGUE_PROBE_SUMMARY: &str = "Developer-triggered dialogue probe.";

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        let broker = OpenAiDialogueBroker::new();
        let broker_status =
            DialogueBrokerStatus::new(broker.provider_kind(), broker.connection_state());

        app.init_resource::<DialogueRateLimitConfig>()
            .init_resource::<DialogueRateLimitState>()
            .init_resource::<DialogueRequestQueue>()
            .init_resource::<PendingDialogueTasks>()
            .init_resource::<DialogueTelemetry>()
            .init_resource::<DialogueTelemetryLog>()
            .insert_resource(broker_status)
            .insert_resource(ActiveDialogueBroker::new(Box::new(broker)))
            .add_message::<DialogueRequestedEvent>()
            .add_message::<DialogueResponseEvent>()
            .add_message::<DialogueRequestFailedEvent>()
            .add_systems(
                Startup,
                (log_dialogue_provider, record_dialogue_broker_status),
            )
            .add_systems(
                Update,
                (
                    handle_dialogue_debug_probe,
                    advance_dialogue_queue_timers,
                    run_dialogue_request_queue,
                    poll_dialogue_tasks, // Poll background tasks for completed requests
                    record_dialogue_telemetry,
                    flush_dialogue_telemetry_log,
                    log_dialogue_events,
                )
                    .chain(),
            );
    }
}

fn handle_dialogue_debug_probe(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut queue: ResMut<DialogueRequestQueue>,
    status: Res<DialogueBrokerStatus>,
    identities: Query<&Identity>,
) {
    if !keyboard.just_pressed(DEBUG_DIALOGUE_PROBE_KEY) {
        return;
    }

    let Some(identity) = identities.iter().next() else {
        warn!("Dialogue probe skipped: no NPC identities available to speak.");
        return;
    };

    let prompt = format!(
        "{} runs a quick dialogue probe for debugging.",
        identity.display_name
    );
    let context = DialogueContext {
        summary: Some(DEBUG_DIALOGUE_PROBE_SUMMARY.to_string()),
        events: Vec::new(),
    };
    let request = DialogueRequest::new(
        identity.id,
        None,
        prompt,
        DialogueTopicHint::Status,
        context,
    );
    let request_id = queue.enqueue(request);

    info!(
        "Queued dialogue probe {} using provider {} ({})",
        request_id.value(),
        status.provider(),
        status.connection_label()
    );
}

fn log_dialogue_provider(status: Res<DialogueBrokerStatus>) {
    match status.connection_state() {
        DialogueConnectionState::Live => {
            info!(
                "Dialogue broker live mode active with provider: {}",
                status.provider()
            );
        }
        DialogueConnectionState::Fallback => {
            warn!(
                "Dialogue broker running in fallback mode with provider: {}. \
                 Set OPENAI_API_KEY to enable live OpenAI responses.",
                status.provider()
            );
        }
    }
    info!(
        "Press {:?} to enqueue a dialogue probe request for quick verification.",
        DEBUG_DIALOGUE_PROBE_KEY
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

fn record_dialogue_broker_status(
    time: Res<Time>,
    status: Res<DialogueBrokerStatus>,
    mut telemetry: ResMut<DialogueTelemetry>,
    mut log: ResMut<DialogueTelemetryLog>,
) {
    let record = DialogueTelemetryRecord {
        occurred_at_seconds: time.elapsed_secs_f64(),
        event: DialogueTelemetryEvent::BrokerStatus(status.to_snapshot()),
    };
    log.push(&record);
    telemetry.push(record);
}
