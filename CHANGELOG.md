# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## 2025-10-19 - S1.16a: Speech Bubble MVP (UI-Based Implementation)
- Created `src/ui/speech_bubble/` module with UI-based speech bubble system using screen-space positioning.
- Implemented `SpeechBubble` component tracking NPC ID, speaker entity, and lifetime timer for UI NodeBundle entities.
- Added `SpeechBubbleSettings` resource exposing configurable lifetime (10s), fade duration (2s), max distance (25u), and font size (20pt).
- Introduced `SpeechBubbleTracker` resource ensuring each NPC has at most one bubble, preventing overlap.
- Created full-screen transparent UI overlay (`SpeechBubbleUiRoot`) parenting all speech bubble nodes for consistent rendering.
- Speech bubbles spawn from `DialogueResponseEvent` as UI nodes with dark semi-transparent backgrounds (Color::srgba(0.1, 0.1, 0.1, 0.85)).
- Positioning system uses `camera.world_to_viewport()` to convert NPC 3D positions to 2D screen coordinates every frame.
- Distance-based culling hides bubbles when NPCs are beyond max_display_distance (default 25 world units).
- Fade-out effect interpolates alpha from 1.0 to 0.0 during final 2 seconds using `lifetime.remaining_secs()`.
- Registered `SpeechBubblePlugin` in `main.rs` after `DialoguePlugin` with startup and update systems.
- **Implementation Note:** Initially prototyped with Text2d world-space billboards, but switched to UI NodeBundle approach for reliable screen-space tracking without billboard rotation complexity.

## 2025-10-18+ - S1.13: Dialogue Broker Verification & Instrumentation
- Restored delivery completion checks so both the sender and recipient must be at their crates before goods transfer, preventing premature task completion during the economy loop.
- Instrumented the dialogue broker with a `DialogueBrokerStatus` resource that logs live vs. fallback mode at startup and surfaces the state to UI/telemetry consumers.
- Added a `broker_status` entry type to `logs/dialogue_history.jsonl`, recording provider mode changes alongside dialogue responses and failures for easier diagnostics.
- Bound `F7` as a dialogue probe hotkey that enqueues a canned NPC prompt through the existing queue so developers can smoke-test OpenAI connectivity on demand.
- Automatically load environment variables from `secrets.env` (when present) before the Bevy app starts, keeping API keys local without manual export steps.

## 2025-10-18 - S1.9-S1.12: Codebase Cleanup & Refactor Support
- Re-ran the standard toolchain (fmt, clippy, check) and captured a fresh responsibility map so economy, dialogue, and NPC modules have a documented baseline before further changes.
- Centralised dialogue and economy literals into named constants/config toggles, promoting shared defaults for OpenAI requests, trade placeholders, and motivation thresholds.
- Split the monolithic economy system into `systems::{spawning, day_prep, task_execution, dialogue}` and broke the dialogue broker into `broker::{mod, config, openai}` to keep each file under 400 lines.
- Removed unused helper functions, redundant imports, and dead types surfaced by `cargo clippy -D warnings -- -D dead_code`, keeping the reorganised modules lint-clean.

## 2025-10-17 - S1.4: Config-Driven Economy Planner Spike
- Added `config/economy.toml` plus `EconomyRegistry`/planner modules so daily trade plans load from data instead of hard-coded loops.
- Replaced the micro trade loop with `prepare_economy_day`/`advance_actor_tasks`, enabling villagers to wait at crates, manufacture goods, and deliver them via queued `ActorTask`s.
- Centralised placeholder meshes/materials through `TradeGoodPlaceholderVisuals`, keeping crate-side goods visible while inventories hold stock.
- Updated economy documentation (`src/economy/README.md`, `docs/tech_notes.md`, `docs/economy_planner_spike.md`) to describe recipes, requests, and task execution flow.

## 2025-10-17 - Fix: Motivation dependency evaluation
- Leisure dopamine rewards no longer mark food dependencies as satisfied, keeping wellbeing data aligned with actual supplies.
- Daily dependency snapshots now queue per-world-day, ensuring bonuses and penalties apply once the calendar advances.
- Dependency matrix checks only credit categories when matching goods are present, and dialogue telemetry logging simplifies directory creation.

## 2025-10-08 - S0.1b: CorePlugin & SimulationClock
- Added `CorePlugin` and `SimulationClock` resource under `src/core/`.
- Registered `CorePlugin` in `main.rs`, ensuring simulation time scaling is centralised.
- Logged startup information to confirm the configured time scale.
- Documented the new module in `src/core/README.md`.

## 2025-10-08 - S0.1c: core_debug Feature & Logging
- Declared a `core_debug` cargo feature that enables per-second simulation tick logging.
- Added a debug timer system that logs scaled time, real delta, and time scale when the feature is active.
- Updated documentation and editor tasks to instruct how to use the debug toggle.

## 2025-10-08 - S0.2a: World Shell Bootstrap
- Introduced `WorldPlugin` with ground plane, directional light, and fly camera controls.
- Implemented WASD + Space/LShift movement and right-mouse look with cursor grab.
- Added `src/world/README.md` describing module responsibilities and usage.
- Registered `WorldPlugin` in `main.rs` to load the scene on startup.
## 2025-10-08 - S0.2b: WorldClock & Configurable Lighting
- Added `WorldTimeSettings` that loads from `config/time.toml` with sane fallbacks.
- Introduced `WorldClock` resource driven by `SimulationClock` to advance time-of-day.
- Implemented systems to rotate the primary sun and adjust ambient light across the day/night cycle.
- Updated world documentation and VS Code tasks to reflect the new configuration knobs.

## 2025-10-08 - S0.3a: Documentation & Automation Sweep
- Refreshed README with time-configuration guidance and cleaned repository layout overview.
- Updated `.agent/docs/arch.md` to include new dependencies and world-time data flow.
- Added `docs/tech_notes.md` for ongoing technical notes and ensured planning artifacts mirror the current state.
- Verified VS Code tasks and configuration files align with the updated workflow.
## 2025-10-08 - S1.1a: NPC Identity & Debug Spawner
- Added npc module with Identity component and NpcIdGenerator resource.
- Registered NpcPlugin and spawned three placeholder NPCs with capsule meshes.
- Documented the module (`src/npc/README.md`) and synced planning artifacts.

## 2025-10-10 - S1.1b: NPC Schedule Scaffold
- Added ScheduleTicker resource to accumulate simulation time and queue schedule ticks.
- Consolidated schedule updates into tick_schedule_state, logging activity transitions at a 5s cadence.
- Updated NPC documentation and planning artifacts to reflect the scheduling scaffold.

## 2025-10-11 - Tooling: Docker Environment Baseline
- Added a multi-stage Dockerfile with dedicated dev, build, and runtime stages for Bevy dependencies.
- Introduced docker-compose.yml to streamline iterative development with mounted sources and cached cargo artifacts.
- Documented container workflows in README.md.

## 2025-10-12 - Tooling: Linux Docker Enablement
- Installed Vulkan headers and Mesa Vulkan drivers in the Docker base/runtime images so Linux hosts can start wgpu without extra packages.
- Added docker-compose.linux.yml override that wires `/dev/dri`, display sockets, and render group membership for desktop runs.
- Documented the Linux workflow across README.md and docs/tech_notes.md.

## 2025-10-12 - S1.3: Dialogue Broker Prototype
- Introduced `DialoguePlugin` with a trait-based broker abstraction, active provider resource, and request queue.
- Implemented global/per-NPC rate limiting, retry backoff, and response/error events for queued dialogue.
- Added structured context events (including trade descriptors) so future providers can cite simulation data.

## 2025-10-12 - S1.4: Micro Trade Loop Spike
- Added `EconomyPlugin` with placeholder professions, inventories, and a daily farmer → miller → blacksmith trade chain.
- Emitted `TradeCompletedEvent` records for production, processing, and exchange steps while logging inventory flow.
- Wired trade exchanges into the dialogue queue, queuing contextualised trade conversations for participating NPCs.

## 2025-10-13 - S1.5: Economy Foundation Blueprint
- Documented the path from the placeholder micro loop to a configurable economy in `docs/economy_blueprint.md`.
- Established plans for an `EconomyRegistry`, `WorkOrderQueue`, and expanded economy event types feeding dialogue and UI.
- Recorded risks, mitigations, and next actions to guide Step 7 implementation tasks.

## 2025-10-13 - S1.6: NPC Locomotion & Profession Crates
- Spawned crate entities for farmer, miller, and blacksmith professions and recorded them in a `ProfessionCrateRegistry`.
- Added an `NpcLocomotion` component/system pair that steers villagers toward crate destinations using scaled simulation time.
- Updated the micro trade loop to halt until each profession reaches its crate, producing visible travel before exchanges fire and logging movement telemetry for future UI hooks.

## 2025-10-14 - Planning: Dependency Matrix & Motivation Spike
- Expanded planning docs (README, TASK.md, BigPicturePlan.md, docs/plan_overview.md, docs/tech_notes.md) to cover the upcoming profession/resource dependency matrix and dopamine-driven motivation system.
- Added S1.7 to the task queue, documenting dopamine decay/gain rules, mood thresholds, and alcohol trade-offs that will influence product quality.
- Updated economy blueprint goals and .agent memory with the dependency matrix requirement so economy configs remain the single source of truth.

## 2025-10-15 - S1.7: NPC Motivation & Wellbeing Spike
- Added `config/motivation.toml` and a `MotivationConfig` resource that loads dopamine caps, decay, gains, thresholds, and alcohol behaviour at startup.
- Introduced `NpcMotivation` components with mood tracking, intoxication/hangover handling, and daily dependency evaluation tied to the new economy dependency matrix.
- Emitted `NpcActivityChangedEvent` signals, rewarding leisure/social moments, marking food satisfaction, and linking trade/dialogue events to motivation rewards.
- Extended the economy module with a placeholder dependency matrix and daily snapshots so wellbeing penalties/rewards reflect tool and food access.

## 2025-10-16 - S1.8: Dialogue Telemetry Persistence
- Added a `DialogueTelemetryLog` resource that writes dialogue responses and failures to `logs/dialogue_history.jsonl` for offline inspection.
- Extended `DialoguePlugin` to initialise the telemetry log and flush it after queue processing so persisted history stays in sync with in-memory records.
- Updated documentation and planning artifacts to describe the new log output and remove the telemetry persistence item from the active backlog.
