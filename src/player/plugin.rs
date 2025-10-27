//! Player plugin wiring interaction systems.
use bevy::prelude::*;

use crate::player::{
    components::PlayerInteractionState,
    systems::{
        cleanup_player_response_window, detect_nearby_npcs, handle_player_interaction_input,
        handle_player_response_buttons, spawn_player_response_window,
    },
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInteractionState>().add_systems(
            Update,
            (
                detect_nearby_npcs,
                handle_player_interaction_input.after(detect_nearby_npcs),
                spawn_player_response_window,
                handle_player_response_buttons.after(spawn_player_response_window),
                cleanup_player_response_window.after(handle_player_response_buttons),
            ),
        );
    }
}
