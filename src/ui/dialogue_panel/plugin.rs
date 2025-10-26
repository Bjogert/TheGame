// src/ui/dialogue_panel/plugin.rs
//
// UiPlugin coordinates dialogue panel systems and resources.

use bevy::prelude::*;

use super::components::{DialoguePanelSettings, DialoguePanelTracker};
use super::systems::{spawn_dialogue_panel, update_dialogue_panel};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        info!("UiPlugin registered");

        app.insert_resource(DialoguePanelSettings::default())
            .insert_resource(DialoguePanelTracker::default())
            .add_systems(
                Update,
                (
                    spawn_dialogue_panel,
                    update_dialogue_panel.after(spawn_dialogue_panel),
                ),
            );
    }
}
