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

## Planning Alignment (2025-10-13)
- Dialogue broker prototype (S1.3) remains in flight; its output must expose trade context once the micro loop lands.
- Introduced a new checkpoint (S1.4) to stand up placeholder professions and crate-based goods immediately after S1.3.
- Full economy milestone (M5) will build on the validated micro loop, expanding resources, pricing, and job depth.

