use bevy::prelude::*;

mod core;
mod dialogue;
mod economy;
mod npc;
mod world;

use crate::{
    core::CorePlugin, dialogue::DialoguePlugin, economy::EconomyPlugin, npc::NpcPlugin,
    world::WorldPlugin,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CorePlugin::default(),
            DialoguePlugin,
            EconomyPlugin,
            WorldPlugin,
            NpcPlugin,
        ))
        .run();
}
