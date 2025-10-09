//! WorldPlugin coordinates environment setup, camera controls, and time-of-day lighting.
use bevy::prelude::*;

use crate::world::{
    systems::{
        fly_camera_mouse_look, fly_camera_translate, spawn_world_environment, update_cursor_grab,
    },
    time::{advance_world_clock, apply_world_lighting, WorldClock, WorldTimeSettings},
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        let time_settings = WorldTimeSettings::load_or_default();
        info!(
            "World time configured: day length {:.2} minutes (sunrise {:.2}, sunset {:.2})",
            time_settings.seconds_per_day / 60.0,
            time_settings.sunrise_fraction,
            time_settings.sunset_fraction
        );

        app.insert_resource(time_settings)
            .insert_resource(WorldClock::new())
            .add_systems(Startup, spawn_world_environment)
            .add_systems(
                Update,
                (
                    advance_world_clock,
                    (
                        update_cursor_grab,
                        fly_camera_mouse_look.after(update_cursor_grab),
                        fly_camera_translate,
                    ),
                    apply_world_lighting.after(advance_world_clock),
                ),
            );
    }
}
