use std::path::Path;

use bevy::prelude::*;

mod core;
mod dialogue;
mod economy;
mod npc;
mod ui;
mod world;

use crate::{
    core::CorePlugin, dialogue::DialoguePlugin, economy::EconomyPlugin, npc::NpcPlugin,
    ui::UiPlugin, world::WorldPlugin,
};

fn main() {
    load_secrets_env();

    App::new()
        .add_plugins((
            DefaultPlugins,
            CorePlugin::default(),
            DialoguePlugin,
            EconomyPlugin,
            WorldPlugin,
            NpcPlugin,
            UiPlugin, // After DialoguePlugin to receive DialogueResponseEvent
        ))
        .run();
}

fn load_secrets_env() {
    const SECRETS_FILE: &str = "secrets.env";

    let path = Path::new(SECRETS_FILE);
    if !path.exists() {
        return;
    }

    if let Err(err) = dotenvy::from_filename(path) {
        eprintln!("Failed to load {}: {}", SECRETS_FILE, err);
    }
}
