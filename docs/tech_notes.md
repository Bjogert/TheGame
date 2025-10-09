# Tech Notes

## World Time & Lighting (S0.2b)
- `WorldTimeSettings` loads from `config/time.toml` at startup. Missing or invalid configs fall back to sane defaults with a logged warning.
- `WorldClock` is driven by the scaled delta from `SimulationClock`, so global time-scale changes automatically affect day length.
- The directional light tagged with `PrimarySun` rotates once per day and blends between configurable day/night intensities.
- Ambient light lerps between day/night colors. Adjust the `[lighting]` values in `config/time.toml` to tune overall scene brightness.
- Cursor grab now uses `CursorOptions`, keeping right-mouse look behaviour consistent with the Bevy 0.17 API.

## Configuration (`config/time.toml`)
```toml
[clock]
day_length_minutes = 10.0
sunrise_fraction = 0.22
sunset_fraction = 0.78
sun_declination_radians = 0.4

[lighting]
noon_lux = 50000.0
night_lux = 5.0
ambient_day = [0.35, 0.35, 0.4]
ambient_night = [0.05, 0.05, 0.1]
```
- `day_length_minutes`: Real-time minutes per in-game day (before time scaling).
- `sunrise_fraction` / `sunset_fraction`: Fractions (0–1) of the day when the sun crosses the horizon.
- `sun_declination_radians`: Tilts the sun path to bias lighting towards dawn/dusk.
- Lighting block controls directional sun intensity and ambient colors.

## NPC Debug Schedules (S1.1a/S1.1b)
- Debug spawner creates three capsule NPCs with unique identities and daily schedules.
- DailySchedule entries use day-fraction start times; update_schedule_state logs when activities change based on WorldClock.
- Schedule data currently lives in code; migrate to config/persistence in later milestones.

