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

## Dialogue Broker Prototype (S1.3)
- Added `DialoguePlugin` with a queue runner, global/per-NPC rate limits, and stubbed `LocalDialogueBroker` responses.
- Requests carry structured context events (including trades) so providers can reference live simulation data.
- Failures emit events for future retry/telemetry handling; context-missing errors surface missing trade history.

## Micro Trade Loop Spike (S1.4)
- Economy module assigns farmer, miller, and blacksmith professions, each with simple inventories.
- A daily loop produces grain, flour, and tool crates, emitting `TradeCompletedEvent` records for each exchange.
- Trade events enqueue dialogue requests with matching context, proving the dialogue hook can react to simulation activity.

## Economy Planning Blueprint (S1.5)
- Reviewed the placeholder micro loop and documented the path to a data-driven economy in `docs/economy_blueprint.md`.
- Step 7 will introduce TOML-driven profession/recipe definitions, persistent inventories, and a work-order queue.
- Dialogue hooks will expand to cover shortages and assignments; economy events will broaden beyond the existing trade event.
- Risks include configuration sprawl and schedule integration complexity; mitigations and open questions are tracked in the blueprint.

## Tooling - Docker Environment (2025-10-11)
- Multi-stage Dockerfile provides `dev`, `build`, and `runtime` targets. Use `docker build --target runtime` for slim release images.
- Base stage now installs Vulkan headers (`libvulkan-dev`) and Mesa Vulkan drivers so Linux hosts can initialise wgpu inside the container without extra host setup beyond the kernel driver.
- `docker-compose.yml` mounts the workspace and caches cargo artifacts; pair it with `docker-compose.linux.yml` to bind `/dev/dri`, compositor sockets, and the render group when running on Linux.
- Runtime stage installs only the dynamic libraries required by wgpu (X11, Wayland, ALSA, udev, Vulkan) to keep distribution images small.
- Hosted CI agents cannot validate Docker commands directly because the Docker
  socket is unavailable inside the sandbox. Expect `docker build` to fail in
  that environment even though the configuration works on a standard developer
  workstation.

