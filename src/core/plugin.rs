//! CorePlugin wires global timing and logging utilities for the simulation.
use bevy::prelude::*;
#[cfg(feature = "core_debug")]
use bevy::time::TimerMode;
use std::time::Duration;

const DEFAULT_TIME_SCALE: f32 = 1.0;
const MIN_TIME_SCALE: f32 = 0.001;

#[cfg(feature = "core_debug")]
#[derive(Resource)]
struct DebugTickTimer {
    timer: Timer,
}

#[cfg(feature = "core_debug")]
impl Default for DebugTickTimer {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
        }
    }
}

/// Tracks scaled simulation time derived from real frame deltas.
#[derive(Resource, Debug)]
pub struct SimulationClock {
    time_scale: f32,
    last_real_delta: Duration,
    last_scaled_delta: Duration,
    elapsed: Duration,
}

impl SimulationClock {
    /// Creates a new clock with the provided time-scale multiplier.
    pub fn new(time_scale: f32) -> Self {
        let clamped = time_scale.max(MIN_TIME_SCALE);
        Self {
            time_scale: clamped,
            last_real_delta: Duration::ZERO,
            last_scaled_delta: Duration::ZERO,
            elapsed: Duration::ZERO,
        }
    }

    /// Sets the time-scale multiplier (clamped to a small positive minimum).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn set_time_scale(&mut self, scale: f32) {
        self.time_scale = scale.max(MIN_TIME_SCALE);
    }

    /// Returns the current time-scale multiplier.
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Last real (unscaled) delta reported by Bevy's Time resource.
    #[cfg_attr(not(feature = "core_debug"), allow(dead_code))]
    pub fn last_real_delta(&self) -> Duration {
        self.last_real_delta
    }

    /// Last scaled delta after applying the multiplier.
    #[cfg_attr(not(feature = "core_debug"), allow(dead_code))]
    pub fn last_scaled_delta(&self) -> Duration {
        self.last_scaled_delta
    }

    /// Returns the total scaled duration elapsed since the clock was initialised.
    #[cfg_attr(not(feature = "core_debug"), allow(dead_code))]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Applies a real delta to the clock, storing both the real and scaled durations.
    pub fn tick(&mut self, real_delta: Duration) {
        self.last_real_delta = real_delta;
        self.last_scaled_delta = real_delta.mul_f32(self.time_scale);
        self.elapsed += self.last_scaled_delta;
    }
}

impl Default for SimulationClock {
    fn default() -> Self {
        Self::new(DEFAULT_TIME_SCALE)
    }
}

/// Registers simulation timing systems and resources.
#[derive(Debug, Clone, Copy)]
pub struct CorePlugin {
    time_scale: f32,
}

impl CorePlugin {
    /// Creates a CorePlugin with the provided time-scale multiplier.
    pub const fn with_time_scale(time_scale: f32) -> Self {
        Self { time_scale }
    }
}

impl Default for CorePlugin {
    fn default() -> Self {
        Self::with_time_scale(DEFAULT_TIME_SCALE)
    }
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SimulationClock::new(self.time_scale))
            .add_systems(Startup, log_startup_time_scale)
            .add_systems(Update, update_simulation_clock);

        #[cfg(feature = "core_debug")]
        {
            app.insert_resource(DebugTickTimer::default())
                .add_systems(Update, log_scaled_ticks);
        }
    }
}

fn update_simulation_clock(mut clock: ResMut<SimulationClock>, time: Res<Time>) {
    clock.tick(time.delta());
}

fn log_startup_time_scale(clock: Res<SimulationClock>) {
    info!(
        "CorePlugin initialised with time scale: {:.3}",
        clock.time_scale()
    );
}

#[cfg(feature = "core_debug")]
fn log_scaled_ticks(mut timer: ResMut<DebugTickTimer>, clock: Res<SimulationClock>) {
    if timer.timer.tick(clock.last_scaled_delta()).just_finished() {
        info!(
            target: "core_debug",
            "Sim elapsed: {:.2}s | scale: {:.3} | real dt: {:.4}s | scaled dt: {:.4}s",
            clock.elapsed().as_secs_f32(),
            clock.time_scale(),
            clock.last_real_delta().as_secs_f32(),
            clock.last_scaled_delta().as_secs_f32(),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_scales_delta_with_multiplier() {
        let mut clock = SimulationClock::new(2.5);
        clock.tick(Duration::from_secs_f32(1.2));

        assert_eq!(clock.time_scale(), 2.5);
        assert_eq!(clock.last_real_delta(), Duration::from_secs_f32(1.2));
        assert_eq!(
            clock.last_scaled_delta(),
            Duration::from_secs_f32(1.2 * 2.5)
        );
        assert_eq!(clock.elapsed(), Duration::from_secs_f32(1.2 * 2.5));
    }

    #[test]
    fn clock_clamps_min_time_scale() {
        let mut clock = SimulationClock::new(0.0);
        assert!((clock.time_scale() - MIN_TIME_SCALE).abs() < f32::EPSILON);

        clock.set_time_scale(-5.0);
        assert!((clock.time_scale() - MIN_TIME_SCALE).abs() < f32::EPSILON);
    }
}
