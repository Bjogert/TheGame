# Tech Notes

## World Time & Lighting (S0.2b)
- WorldTimeSettings loads from config/time.toml at startup; missing or invalid configs fall back to defaults with a warning.
- WorldClock is driven by the scaled delta from SimulationClock, keeping day length responsive to global time-scale changes.
- The directional light tagged with PrimarySun rotates once per day and blends between configurable day/night intensities.
- Ambient light lerps between day/night colors. Adjust the [lighting] values in config/time.toml to tune overall scene brightness.
- Cursor grab uses CursorOptions, keeping right-mouse look behaviour consistent with the Bevy 0.17 API.

## NPC Debug Schedules (S1.1a/S1.1b)
- Debug spawner creates three capsule NPCs with unique identities and daily schedules.
- ScheduleTicker accumulates simulation time (default 5 seconds) and triggers activity transitions driven by WorldClock.
- DailySchedule entries use day-fraction start times; logs surface transitions for visibility.
- Schedule data currently lives in code; migrate to config/persistence in later milestones.
