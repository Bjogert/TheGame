//! World time configuration, clock resource, and lighting systems.
use std::{f32::consts::TAU, fs, path::Path};

use bevy::prelude::*;
use serde::Deserialize;

use crate::core::plugin::SimulationClock;
use crate::world::components::PrimarySun;

const CONFIG_PATH: &str = "config/time.toml";

#[derive(Debug, Clone, Deserialize, Default)]
struct RawTimeConfig {
    #[serde(default)]
    clock: RawClockSection,
    #[serde(default)]
    lighting: RawLightingSection,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawClockSection {
    day_length_minutes: f32,
    sunrise_fraction: f32,
    sunset_fraction: f32,
    sun_declination_radians: f32,
}

impl Default for RawClockSection {
    fn default() -> Self {
        Self {
            day_length_minutes: 10.0,
            sunrise_fraction: 0.22,
            sunset_fraction: 0.78,
            sun_declination_radians: 0.4,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
struct RawLightingSection {
    noon_lux: f32,
    night_lux: f32,
    ambient_day: [f32; 3],
    ambient_night: [f32; 3],
}

impl Default for RawLightingSection {
    fn default() -> Self {
        Self {
            noon_lux: 50_000.0,
            night_lux: 5.0,
            ambient_day: [0.35, 0.35, 0.4],
            ambient_night: [0.05, 0.05, 0.1],
        }
    }
}

/// Tunable parameters describing how the world clock behaves.
#[derive(Resource, Debug, Clone)]
pub struct WorldTimeSettings {
    pub seconds_per_day: f32,
    pub sunrise_fraction: f32,
    pub sunset_fraction: f32,
    pub sun_declination: f32,
    pub noon_lux: f32,
    pub night_lux: f32,
    pub ambient_day: Vec3,
    pub ambient_night: Vec3,
}

impl WorldTimeSettings {
    pub fn load_or_default() -> Self {
        let path = Path::new(CONFIG_PATH);
        match fs::read_to_string(path) {
            Ok(data) => match toml::from_str::<RawTimeConfig>(&data) {
                Ok(raw) => raw.into(),
                Err(err) => {
                    warn!(
                        "Failed to parse {} ({}). Falling back to defaults.",
                        CONFIG_PATH, err
                    );
                    RawTimeConfig::default().into()
                }
            },
            Err(err) => {
                warn!(
                    "Failed to read {} ({}). Falling back to defaults.",
                    CONFIG_PATH, err
                );
                RawTimeConfig::default().into()
            }
        }
    }
}

impl From<RawTimeConfig> for WorldTimeSettings {
    fn from(value: RawTimeConfig) -> Self {
        let clock = value.clock;
        let lighting = value.lighting;

        let seconds_per_day = (clock.day_length_minutes.max(0.1)) * 60.0;
        let sunrise = clock.sunrise_fraction.clamp(0.0, 1.0);
        let sunset = clock.sunset_fraction.clamp(0.0, 1.0);
        let (sunrise, sunset) = if sunrise == sunset {
            (sunrise, (sunrise + 0.5) % 1.0)
        } else {
            (sunrise.min(sunset), sunrise.max(sunset))
        };

        Self {
            seconds_per_day,
            sunrise_fraction: sunrise,
            sunset_fraction: sunset,
            sun_declination: clock.sun_declination_radians,
            noon_lux: lighting.noon_lux.max(lighting.night_lux),
            night_lux: lighting.night_lux.max(0.0),
            ambient_day: Vec3::new(
                lighting.ambient_day[0],
                lighting.ambient_day[1],
                lighting.ambient_day[2],
            ),
            ambient_night: Vec3::new(
                lighting.ambient_night[0],
                lighting.ambient_night[1],
                lighting.ambient_night[2],
            ),
        }
    }
}

/// Runtime state for the world clock.
#[derive(Resource, Debug)]
pub struct WorldClock {
    time_of_day: f32,
    day_count: u64,
}

impl WorldClock {
    pub fn new() -> Self {
        Self {
            time_of_day: 0.0,
            day_count: 0,
        }
    }

    pub fn time_of_day(&self) -> f32 {
        self.time_of_day
    }

    pub fn day_count(&self) -> u64 {
        self.day_count
    }

    fn tick(&mut self, delta_seconds: f32, settings: &WorldTimeSettings) {
        let mut fraction = delta_seconds / settings.seconds_per_day;
        if fraction.is_nan() || !fraction.is_finite() {
            fraction = 0.0;
        }
        self.time_of_day = (self.time_of_day + fraction) % 1.0;
        if self.time_of_day < fraction {
            self.day_count = self.day_count.saturating_add(1);
        }
    }
}

/// Advances the world clock based on the SimulationClock delta.
pub fn advance_world_clock(
    mut clock: ResMut<WorldClock>,
    settings: Res<WorldTimeSettings>,
    simulation_clock: Res<SimulationClock>,
) {
    let delta = simulation_clock.last_scaled_delta().as_secs_f32();
    clock.tick(delta, &settings);
}

/// Applies time-of-day lighting to the primary sun and ambient light.
pub fn apply_world_lighting(
    clock: Res<WorldClock>,
    settings: Res<WorldTimeSettings>,
    mut ambient: ResMut<AmbientLight>,
    mut sun_query: Query<(&PrimarySun, &mut Transform, &mut DirectionalLight)>,
) {
    let day_fraction = clock.time_of_day();
    let sun_angle = (day_fraction - 0.25) * TAU;
    let declination = settings.sun_declination;

    let rotation = Quat::from_euler(EulerRot::ZYX, 0.0, declination, sun_angle).normalize();

    let daylight_factor = {
        let daylight_span = settings.sunset_fraction - settings.sunrise_fraction;
        if daylight_span <= 0.0 {
            1.0
        } else {
            let mut t = day_fraction;
            if t < settings.sunrise_fraction {
                t += 1.0;
            }
            let offset = (t - settings.sunrise_fraction) % 1.0;
            let normalized = (offset / daylight_span).clamp(0.0, 1.0);
            normalized.sin().max(0.0)
        }
    };

    let intensity =
        settings.night_lux + (settings.noon_lux - settings.night_lux) * daylight_factor.powf(1.5);

    let ambient_vec = settings
        .ambient_night
        .lerp(settings.ambient_day, daylight_factor);
    ambient.color = Color::linear_rgb(ambient_vec.x, ambient_vec.y, ambient_vec.z);

    for (_, mut transform, mut light) in sun_query.iter_mut() {
        transform.rotation = rotation;
        light.illuminance = intensity;
    }
}
