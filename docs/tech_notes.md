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
- Managed LLM APIs (OpenAI) chosen for rapid iteration; abstract via DialogueBroker.
- Rate limiting plan: global 60 req/min bucket + per-NPC 30s cooldown with queued requests.
- Prompt context uses Identity, ScheduleState, WorldClock; future fields include relationships, goals, mood.
- See docs/dialogue_research.md for provider comparison and next steps.

## Dialogue Broker Prototype (S1.3)
- `DialoguePlugin` now initialises the queue, rate-limit resources, and logs both dialogue responses and retryable failures to keep telemetry flowing while the OpenAI stub is active.
- `OpenAiDialogueBroker` validates prompts/context, emits explicit provider errors, and expands responses with summaries, targets, and trade history snippets.
- Warning debt cleared: every exported type is exercised either by runtime systems or focused tests, keeping `cargo check` and `cargo clippy -D warnings` clean.
- Dialogue module tests explicitly import the `DialogueBroker` trait so the OpenAI stub’s `process` path remains covered without re-exporting the trait publicly.

## Micro Trade Loop Spike (S1.4)
- Economy module assigns farmer, miller, and blacksmith professions, each with simple inventories.
- A daily loop produces grain, flour, and tool crates, emitting `TradeCompletedEvent` records for each exchange.
- Trade events enqueue dialogue requests with matching context, proving the dialogue hook can react to simulation activity.

## Economy Planning Blueprint (S1.5)
- Reviewed the placeholder micro loop and documented the path to a data-driven economy in `docs/economy_blueprint.md`.
- Step 7 will introduce TOML-driven profession/recipe definitions, persistent inventories, and a work-order queue.
- Dialogue hooks will expand to cover shortages and assignments; economy events will broaden beyond the existing trade event.
- Risks include configuration sprawl and schedule integration complexity; mitigations and open questions are tracked in the blueprint.

## NPC Locomotion & Profession Crates (S1.6)
- `ProfessionCrateRegistry` tracks dedicated crate entities spawned at startup; each profession now has a visible work location.
- `NpcLocomotion` moves villagers toward their assigned crate along the ground plane, logging travel start and arrival for telemetry.
- The micro trade loop waits until the farmer, miller, and blacksmith reach their crates before processing daily exchanges, giving trades a visible lead-in.
- Movement destinations currently rely on straight-line travel; pathfinding and avoidance remain future work once richer level geometry exists.

## NPC Motivation & Wellbeing Spike (S1.7)
- Motivation will be represented by a bounded dopamine meter tracked per NPC, with configurable gains from task completion, social interaction, leisure, or other satisfying beats.
- Natural decay and situational penalties (missed work, isolation) drain dopamine; thresholds map to mood states (content, tired, depressed) that influence schedules, dialogue tone, and productivity.
- Alcohol and similar coping tools provide a temporary dopamine boost but flag a hangover crash that dips the meter below baseline and applies output-quality penalties while intoxicated.
- The motivation system integrates with the economy dependency matrix so wellbeing modifiers can reference resource access (food, tools) when determining long-term happiness.

## Tooling - Docker Environment (2025-10-11)
- Multi-stage Dockerfile provides `dev`, `build`, and `runtime` targets. Use `docker build --target runtime` for slim release images.
- Base stage now installs Vulkan headers (`libvulkan-dev`) and Mesa Vulkan drivers so Linux hosts can initialise wgpu inside the container without extra host setup beyond the kernel driver.
- `docker-compose.yml` mounts the workspace and caches cargo artifacts; pair it with `docker-compose.linux.yml` to bind `/dev/dri`, compositor sockets, and the render group when running on Linux.
- Runtime stage installs only the dynamic libraries required by wgpu (X11, Wayland, ALSA, udev, Vulkan) to keep distribution images small.
- Hosted CI agents cannot validate Docker commands directly because the Docker
  socket is unavailable inside the sandbox. Expect `docker build` to fail in
  that environment even though the configuration works on a standard developer
  workstation.

