use bevy::prelude::*;

mod core;
mod dialogue;
mod npc;
mod world;

use crate::{core::CorePlugin, dialogue::DialoguePlugin, npc::NpcPlugin, world::WorldPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CorePlugin::default(),
            WorldPlugin,
            NpcPlugin,
            DialoguePlugin::default(),
        ))
        .run();
}
