//! Dialogue plugin wiring queue resources and systems.
use bevy::prelude::*;

use super::{
    broker::LocalDialogueBroker,
    events::{DialogueRequestFailedEvent, DialogueResponseEvent},
    queue::{
        advance_dialogue_queue_timers, run_dialogue_request_queue, ActiveDialogueBroker,
        DialogueRateLimitConfig, DialogueRateLimitState, DialogueRequestQueue,
    },
};

pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueRateLimitConfig>()
            .init_resource::<DialogueRateLimitState>()
            .init_resource::<DialogueRequestQueue>()
            .insert_resource(ActiveDialogueBroker::new(Box::new(
                LocalDialogueBroker::new(),
            )))
            .add_event::<DialogueResponseEvent>()
            .add_event::<DialogueRequestFailedEvent>()
            .add_systems(Startup, log_dialogue_provider)
            .add_systems(
                Update,
                (advance_dialogue_queue_timers, run_dialogue_request_queue).chain(),
            );
    }
}

fn log_dialogue_provider(broker: Res<ActiveDialogueBroker>) {
    info!(
        "DialoguePlugin initialised with provider: {}",
        broker.provider_kind()
    );
}
