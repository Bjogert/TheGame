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

## Dialogue Scaffolding (S1.2)
- Managed LLM APIs (OpenAI/Anthropic) chosen for rapid iteration; abstract via DialogueBroker.
- Rate limiting plan: global 60 req/min bucket + per-NPC 30s cooldown with queued requests.
- Prompt context uses Identity, ScheduleState, WorldClock; future fields include relationships, goals, mood.
- See docs/dialogue_research.md for provider comparison and next steps.

## Tooling - Docker Environment (2025-10-11)
- Multi-stage Dockerfile provides `dev`, `build`, and `runtime` targets. Use `docker build --target runtime` for slim release images.
- `docker-compose.yml` mounts the workspace and caches cargo artifacts; pass display/GPU devices on demand when running the Bevy client.
- Runtime stage installs only the dynamic libraries required by wgpu (X11, Wayland, ALSA, udev) to keep distribution images small.
- Hosted CI agents cannot validate Docker commands directly because the Docker
  socket is unavailable inside the sandbox. Expect `docker build` to fail in
  that environment even though the configuration works on a standard developer
  workstation.

