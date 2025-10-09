use bevy::prelude::*;

mod core;
mod npc;
mod world;

use crate::{core::CorePlugin, npc::NpcPlugin, world::WorldPlugin};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            CorePlugin::default(),
            WorldPlugin,
            NpcPlugin,
        ))
        .run();
}
