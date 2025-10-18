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

## Dialogue Telemetry Persistence (S1.8)
- `DialogueTelemetryLog` now mirrors the in-memory telemetry ring buffer to `logs/dialogue_history.jsonl`, appending JSON lines for both responses and failures.
- The log directory is created on demand; each entry stores provider, speaker/target labels, request ids, and error metadata so UI tooling or offline analysis can replay recent conversations.
- Persistence runs immediately after the request queue processes an entry, keeping the JSONL file aligned with the records exposed to UI systems.

## Baseline Verification & Responsibility Map (S1.9)
- Re-ran `cargo fmt`, `cargo clippy -D warnings`, and `cargo check --all-targets` to confirm the refactored modules compile cleanly on the documented toolchain before starting the cleanup pass.
- Captured an updated responsibility map for core, world, NPC, dialogue, and economy modules in the planning docs so future refactors have a reliable reference.
- Recorded environment caveats (Wayland pkg-config requirement) alongside the baseline to explain why hosted containers still block the linting toolchain.

## Economy & Dialogue Literal Audit (S1.10)
- Consolidated OpenAI defaults (`DEFAULT_MODEL`, `DEFAULT_TIMEOUT_SECS`, `DEFAULT_TEMPERATURE`, etc.) into `broker/config.rs`, keeping environment overrides consistent.
- Moved trade placeholder offsets and profession labels (`BLACKSMITH_NAME`, `MILLER_NAME`) into module-level `const` blocks so systems reuse a single source of truth.
- Surfaced locomotion tolerances, dopamine thresholds, and other tuning knobs in configs/consts called out in README/TASK.md to discourage future magic numbers.

## Systems Modularisation (S1.11)
- Split the 800+ line `economy/systems.rs` into focused modules: `spawning` handles crate/entity setup, `day_prep` plans the day, `task_execution` advances queues, and `dialogue` bridges planner output into the broker.
- Extracted the dialogue broker into `broker/mod.rs`, `broker/config.rs`, and `broker/openai.rs`, isolating HTTP/client wiring from trait definitions and making room for alternate providers.
- Updated `mod.rs` re-exports so external call sites keep their previous import paths while benefiting from smaller files.

## Dead Code Cleanup (S1.12)
- Enabled `cargo clippy -D warnings -- -D dead_code` during the sweep; removed unused helper functions, redundant imports, and stale type aliases highlighted by the lint run.
- Cleared lingering `allow(dead_code)` markers that were originally protecting future config hooks now backed by real constants/config entries.
- Verified telemetry/event wiring after removals to ensure NPC motivation, trade completion, and dialogue pipelines still emit the same signals.

## Config-Driven Micro Trade Planner (S1.4)
- `EconomyRegistry` reads recipes and daily requests from `config/economy.toml`, replacing the hard-coded farmer → miller → blacksmith loop.
- `prepare_economy_day` converts unmet needs into per-profession `ActorTask` queues (`WaitForGood`, `Manufacture`, `Deliver`) so villagers only act when inputs are present.
- `advance_actor_tasks` waits for NPCs to reach their crates, performs manufacture/delivery steps, emits `TradeCompletedEvent`/dialogue prompts, and keeps crate-side placeholders synced with inventory counts.
- Deliveries now confirm both the sender and recipient are stationed at their respective crates before completing, so exchanges remain visible and avoid phantom transfers.
- Task execution leaves goods visible beside crates until they are consumed or traded away, giving a quick read on local stock levels.

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
- `config/motivation.toml` loads dopamine caps, decay, gains, mood thresholds, and alcohol behaviour at startup. Adjusting the file changes runtime tuning after a restart.
- `NpcMotivation` tracks dopamine, mood state, intoxication timers, and hangovers. Baseline decay runs each tick; mood shifts are logged so designers can watch wellbeing trends.
- Activity transitions emit `NpcActivityChangedEvent`. Leisure keywords (e.g., supper, tavern) award dopamine and can trigger alcohol boosts that later crash into hangovers, but they no longer spoof food dependency satisfaction.
- Trade completions and dialogue responses feed rewards into motivation systems. While intoxicated or hungover, work rewards are reduced by the configured quality penalty.
- Daily dependency snapshots are evaluated once the world day advances, so bonuses and penalties reflect the previous day's access to tools, food, and other categories.
- `EconomyDependencyMatrix` and `ProfessionDependencyUpdateEvent` expose which wellbeing categories each profession satisfied. Categories only count when matching goods exist in inventory; missing requirements apply penalties while complete coverage grants a daily bonus.

## Tooling - Docker Environment (2025-10-11)
- Multi-stage Dockerfile provides `dev`, `build`, and `runtime` targets. Use `docker build --target runtime` for slim release images.
- Base stage now installs Vulkan headers (`libvulkan-dev`) and Mesa Vulkan drivers so Linux hosts can initialise wgpu inside the container without extra host setup beyond the kernel driver.
- `docker-compose.yml` mounts the workspace and caches cargo artifacts; pair it with `docker-compose.linux.yml` to bind `/dev/dri`, compositor sockets, and the render group when running on Linux.
- Runtime stage installs only the dynamic libraries required by wgpu (X11, Wayland, ALSA, udev, Vulkan) to keep distribution images small.
- Hosted CI agents cannot validate Docker commands directly because the Docker
  socket is unavailable inside the sandbox. Expect `docker build` to fail in
  that environment even though the configuration works on a standard developer
  workstation.

## Tooling - Wayland pkg-config Gap (2025-10-16)
- `cargo clippy -- -D warnings` and `cargo check --all-targets` currently fail inside the hosted container because the `wayland-client` system package is missing. The `wayland-sys` crate calls out the absent `wayland-client.pc` via pkg-config.
- Installing the Wayland development libraries (e.g., `libwayland-dev` on Debian/Ubuntu) or exposing the correct `PKG_CONFIG_PATH` should unblock the build. Until then, treat the failure as an environment limitation rather than a code regression.



