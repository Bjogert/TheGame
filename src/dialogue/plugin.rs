//! Dialogue plugin wires the broker registry and queue systems.
use bevy::prelude::*;

use super::{
    broker::{DialogueBrokerRegistry, DialogueProvider, LocalEchoBroker},
    queue::DialogueRequestQueue,
    systems::run_dialogue_queue,
};

#[derive(Default)]
pub struct DialoguePlugin;

impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DialogueRequestQueue>()
            .init_resource::<DialogueBrokerRegistry>()
            .add_systems(Startup, register_default_brokers)
            .add_systems(Update, run_dialogue_queue);
    }
}

fn register_default_brokers(mut registry: ResMut<DialogueBrokerRegistry>) {
    if !registry.is_registered(DialogueProvider::Local) {
        registry.register(Box::new(LocalEchoBroker::default()));
    }
}
