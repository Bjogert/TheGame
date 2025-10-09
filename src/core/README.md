# Core Module

The `core` module owns global application scaffolding that every other plugin depends on.

## Contents
- `CorePlugin` registers foundational systems/resources such as the `SimulationClock`.
- `SimulationClock` converts real frame deltas into scaled simulation time, allowing the rest of the game to run faster/slower than real time.
- Startup logging confirms the configured time scale when the application launches.

## Integration Notes
- Add `CorePlugin` before any other project plugins so its resources are available globally:
  ```rust
  App::new()
      .add_plugins((DefaultPlugins, CorePlugin::default()))
      .run();
  ```
- Use `Res<SimulationClock>` in downstream systems when simulation-scaled delta or elapsed time is required.
- Clamp time-scale values using `SimulationClock::set_time_scale` to avoid zero/negative scaling.
- Enable the optional `core_debug` feature (`cargo run --features core_debug`) to log scaled ticks once per second. This is off by default to keep logs clean.

## Follow-ups
- Wire the clock into configuration once `/config/time.toml` lands (S0.2b).
- Emit metrics or events when the time scale changes to aid debugging.
