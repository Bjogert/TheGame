# TASK PLAN

_Last updated: 2025-10-10 (UTC). This file explains the step-by-step execution plan so humans can follow the flow easily. For day-to-day coordination between agents, see `.agent/tasks.yaml`._

---

## Step S0.1a - Confirm Toolchain & Baseline Run
**Goal:** Ensure the developer environment is ready before touching code.

- [x] Record the active Rust toolchain (`rustup show`) and confirm `rustfmt` + `clippy` components are installed.
- [x] Run `cargo fmt`, `cargo clippy -- -D warnings`, and `cargo check --all-targets` to verify the scaffold compiles cleanly.
- [x] Launch `cargo run` to confirm the Bevy window opens without errors.
- [x] Capture outcomes (toolchain version, any hiccups) in `.agent/ai_memory.V.1.yaml` under the current step.
- **Outcome:** Stable toolchain `rustc 1.90.0 (1159e78c4 2025-09-14)` with `rustfmt` and `clippy` installed. `cargo run` launches successfully; CLI sessions must close the window manually or they will time out after ~5 minutes. WGPU prints expected warnings about missing Vulkan validation layers.
- **Exit criteria:** All commands succeed; initial behaviour is documented.

---

## Step S0.1b - Introduce CorePlugin & SimulationClock
**Goal:** Replace the bare `main.rs` with a modular core that manages time scaling.

- [x] Add `src/core/plugin.rs` implementing `CorePlugin`, a `SimulationClock` resource, and startup logging for sanity.
- [x] Adjust `src/main.rs` to register `CorePlugin` and keep the app compiling.
- [x] Consider whether Bevy's built-in `Time<Virtual>` satisfies requirements; document the decision in `ai_memory`.
- [x] Add a unit test or simple assertion (where feasible) ensuring `SimulationClock` applies the time-scale multiplier.
- **Outcome:** Custom `SimulationClock` resource wraps Bevy's `Time` to provide clamped, scaled deltas. Kept `set_time_scale` for future config usage, adding a non-test `allow(dead_code)` until wiring arrives. Unit tests confirm scaling and clamping. `CorePlugin` logs configured scale on startup.
- **Exit criteria:** App compiles and runs with the new plugin registered; behaviour mirrors the previous baseline.

---

## Step S0.1c - Add Debug Feature & Documentation Hooks
**Goal:** Provide diagnostic logging without polluting release builds.

- [x] Introduce a `core_debug` feature in `Cargo.toml`.
- [x] Gate a system that logs the scaled tick once per second behind `core_debug`.
- [x] Update relevant docs: `src/core/README.md`, `CHANGELOG.md`, `.agent/ai_memory.V.1.yaml`, and `.agent/tasks.yaml` (mark S0.1b done, S0.1c in progress).
- [x] Expose the feature in VS Code tasks (e.g., `cargo run --features core_debug`).
- **Outcome:** `core_debug` feature installs a repeating timer that logs simulation elapsed time, real vs. scaled deltas, and the current time-scale multiplier once per second. VS Code commands now forward `-- -D warnings` to Clippy and include feature-aware runs. Docs and planning artifacts capture the toggle and next steps.
- **Exit criteria:** Running with `--features core_debug` prints scaled tick logs; docs explain how to toggle the feature.

---

## Step S0.2a - Bootstrap World Shell
**Goal:** Create a simple world scene as groundwork for future systems.

- [x] Scaffold `src/world/` with modules for components, systems, and `WorldPlugin`.
- [x] Spawn a ground plane, directional light, and placeholder camera controller (keyboard + mouse) to inspect the scene.
- [x] Ensure assets/configs loaded are documented (e.g., primitives vs. GLTF placeholders).
- **Outcome:** `WorldPlugin` now spawns a 200200 plane, directional light, and a fly camera with WASD + Space/LShift movement and right-mouse look (cursor grab toggled automatically). Documentation covers usage and future follow-ups.
- **Exit criteria:** Running the app shows the simple environment and allows basic camera movement.

---

## Step S0.2b - WorldClock & Config Wiring
**Goal:** Make day/night pacing configurable.

- [x] Define a `WorldClock` resource that tracks time-of-day and season placeholders.
- [x] Introduce `/config/time.toml` (or similar) to load time-scale parameters at startup.
- [x] Update systems to consume `SimulationClock` + `WorldClock` to progress the environment.
- [x] Document configuration knobs in `docs/tech_notes.md` and `src/world/README.md`.
- **Outcome:** `WorldTimeSettings` now loads from `config/time.toml`, feeding `WorldClock` and the sun/ambient lighting system. Directional light rotation/intensity and ambient colors respond to the clock, and cursor grabbing uses the new `CursorOptions` path. Docs describe tuning knobs and defaults.
- **Exit criteria:** Adjusting config values changes the in-app day/night speed; documentation reflects the behaviour.

---

## Step S0.3a - Documentation & Automation Sweep
**Goal:** Keep knowledge in sync after the initial milestone.

- [x] Refresh README.md, docs/tech_notes.md, and .agent/docs/arch.md with the implemented systems.
- [x] Append a CHANGELOG.md entry summarising the whole S0 milestone.
- [x] Update .agent/ai_memory.V.1.yaml (promote durable lessons) and clean up .agent/tasks.yaml (mark completed steps, queue next ones).
- [x] Verify VS Code tasks cover new workflows (core debug run, world config reloads, etc.).
- **Outcome:** Docs now explain time configuration, architecture references the world clock pipeline, planning artifacts are aligned, and automation tasks remain accurate.
- **Exit criteria:** Documentation matches the current codebase; automation tasks feel ergonomic; milestone S0 ready for review.
---

## Step S1.1a - NPC Identity & Debug Spawner
**Goal:** Introduce a minimal NPC module with identity data and placeholder entities.

- [x] Create `src/npc/` module (components, plugin, systems) and register `NpcPlugin`.
- [x] Implement `Identity` component and ID generator resource.
- [x] Spawn placeholder NPCs with simple meshes positioned on the ground plane.
- [x] Document the NPC module and update CHANGELOG/planning artifacts.
- **Exit criteria:** Debug NPCs appear in the world with unique identities and documentation reflects the new module.

---

## Step S1.1b - NPC Schedule Scaffold
**Goal:** Introduce a stubbed daily schedule system and verify NPC activity transitions.

- [x] Design schedule data structures and determine how they tick (resource-driven via ScheduleTicker).
- [x] Implement a minimal schedule update system that logs activity changes.
- [ ] Add debug assertions/tests for schedule transitions (deferred).
- [x] Document schedule behaviour and update planning artifacts.
- **Outcome:** NPCs now transition between activities every ~5s of simulation time, and documentation/planning files reflect the scheduling scaffold.
- **Exit criteria:** NPC tick cadence is observable in logs and ready for future needs integration.

---

## Step S1.2 - Dialogue Scaffolding Research
**Overview:** Gather information about which AI chat services to consider, how fast they can talk, and what information they need from our villagers before we build anything.
**Goal:** Capture LLM provider options, rate limiting strategy, and prompt scaffolding requirements.

- [x] Enumerate managed vs. local LLM providers with pros/cons.
- [x] Define global/per-NPC rate limiting strategy and queue behaviour.
- [x] Draft prompt template and identify required simulation context.
- [x] Document findings in docs/dialogue_research.md and update project docs.
- **Outcome:** Research documented; README/tech notes updated, planning artifacts point to the upcoming dialogue broker prototype.
- **Exit criteria:** Dialogue research summary exists and informs implementation work.

---

## Step S1.3 - Dialogue Broker Prototype
**Goal:** Deliver the first broker implementation capable of queuing dialogue requests with rate limiting and context awareness.

- [x] Define a `DialogueBroker` trait and provider enum with a local placeholder implementation.
- [x] Add a request queue resource with global and per-NPC cooldown tracking plus retry backoff.
- [x] Emit success/failure events and surface structured context (including trade descriptors) for future providers.
- [x] Register `DialoguePlugin` in `main.rs` and document the behaviour.
- **Outcome:** Dialogue requests now flow through a managed queue with rate limits, context validation, and telemetry hooks.
- **Exit criteria:** Broker processes local requests, respects cooldowns, and documents the abstraction.

---

## Step S1.4 - Config-driven Micro Trade Planner
**Goal:** Replace the hard-coded trade loop with a data-driven planner so professions act naturally and can scale with new recipes.

- [x] Load recipes and daily requests from `config/economy.toml` via `EconomyRegistry`.
- [x] Derive per-profession `ActorTask` queues (`WaitForGood`, `Manufacture`, `Deliver`) when needs are detected.
- [x] Execute tasks only after villagers reach their crates, emit `TradeCompletedEvent` + dialogue prompts on delivery, and keep crate placeholders in sync with inventory.
- **Outcome:** Villagers plan each workday from data, wait for partners before trades, and the loop now supports additional professions/recipes without code changes.
- **Exit criteria:** Planner queues populate from config, goods appear/disappear beside crates with inventory counts, and trade deliveries emit telemetry/dialogue after both actors are present.

---

## Step S1.5 - Economy Foundation Blueprint
**Goal:** Capture the design for evolving the placeholder micro loop into the first configurable economy slice.

- [x] Survey the current economy, dialogue, and NPC integrations to identify extension points.
- [x] Define resources (EconomyRegistry, WorkOrderQueue), config approach, and event taxonomy for Step 7.
- [x] Record risks, mitigations, and open questions plus next actions leading into implementation.
- **Outcome:** `docs/economy_blueprint.md` documents the data-driven economy plan, informing Step 7 backlog items and calling for a profession/resource dependency matrix to keep balancing readable.
- **Exit criteria:** Blueprint published, supporting docs updated, and planning artifacts reflect the new direction.

---

## Step S1.6 - NPC Locomotion & Profession Crates
**Goal:** Give NPCs tangible work locations and simple pathing so their schedules produce visible movement.

- [x] Spawn placeholder crate entities for each profession (farmer, miller, blacksmith) in consistent world positions.
- [x] Extend schedules/work orders to assign NPCs a destination crate before performing trade or craft actions.
- [x] Add a lightweight locomotion system that moves NPCs toward their assigned crate with speed clamping and arrival tolerance.
- [x] Emit telemetry/logs for movement start/complete events and capture follow-ups in planning docs.
- **Outcome:** NPCs walk from their idle position to the crate that matches their current task, creating clear visual activity loops ahead of future field/workshop assets. Movement is currently gated so the daily trade loop waits until everyone reaches their crate, giving trades a visible lead-in.
- **Exit criteria:** When the simulation runs, each profession NPC visibly travels to its crate before executing the trade loop; logs/documentation record the new behaviour and remaining movement limitations.

---

## Step S1.7 - NPC Motivation & Wellbeing Spike
**Goal:** Prototype a dopamine-style motivation meter that reacts to work, socialising, and coping mechanisms so behaviour and product quality feel grounded.

- [x] Add a per-NPC dopamine resource with configurable caps, baseline decay, and gains tied to completed tasks, social interactions, and leisure events.
- [x] Map dopamine thresholds to mood states (content, tired, depressed) that influence schedule modifiers, dialogue tone, and production efficiency.
- [x] Model alcohol as a temporary dopamine boost with intoxication penalties (reduced work rewards) and a hangover crash that dips below the starting point.
- [x] Connect the motivation data to the economy dependency matrix so resource access (food, tools, housing) can influence long-term wellbeing, evaluating snapshots after the day rolls over instead of relying on leisure to spoof needs.
- [x] Update planning docs, tech notes, and telemetry plans to cover the new resource and its tuning knobs.
- **Exit criteria:** Motivation metrics surface in telemetry/docs, mood thresholds and alcohol side effects are documented, and follow-up tasks for full integration are queued.

---

## Step S1.8 - Dialogue Telemetry Persistence
**Goal:** Capture dialogue telemetry in a persisted log so UI tooling and offline analysis can review recent conversations.

- [x] Introduce a `DialogueTelemetryLog` resource that mirrors the in-memory telemetry buffer to disk.
- [x] Flush dialogue responses and failures to `logs/dialogue_history.jsonl` after each queue tick, guarding the directory creation.
- [x] Extend documentation (README, module README, tech notes) with instructions for inspecting or resetting the log.
- [x] Update planning artifacts (`CHANGELOG.md`, `.agent/tasks.yaml`, `.agent/ai_memory.V.1.yaml`, `BigPicturePlan.md`, `docs/plan_overview.md`, `TASK.md`) to reflect the completed persistence step and upcoming UI/dependency matrix work.
- **Outcome:** JSONL telemetry history accumulates under `logs/`, kept in sync with the ring buffer for UI consumers and ready for external tooling.
- **Exit criteria:** Running the app produces dialogue entries in `logs/dialogue_history.jsonl`; docs describe the log and the backlog no longer lists telemetry persistence as pending.

---

## Step S1.9 - Baseline Verification & Responsibility Map
**Goal:** Confirm the refactored project compiles cleanly and document current module responsibilities before further cleanup.

- [x] Run `cargo fmt`, `cargo clippy -D warnings`, and `cargo check --all-targets` on the refreshed toolchain to re-establish the baseline.
- [x] Capture a module responsibility snapshot (core, world, npc, dialogue, economy) in the planning docs for future reference.
- [x] Record environment caveats (Wayland pkg-config gap) alongside the baseline to explain current CI/tooling limitations.
- **Outcome:** Toolchain checks pass locally, the current behaviour map is documented, and future refactors have a dependable reference point.
- **Exit criteria:** Baseline commands succeed and the planning docs describe how major plugins interact today.

---

## Step S1.10 - Economy & Dialogue Literal Audit
**Goal:** Replace scattered magic numbers with named constants or config entries.

- [x] Promote OpenAI defaults (model, timeout, token caps, temperature) into `dialogue::broker::config` so environment overrides share one source of truth.
- [x] Centralise trade placeholder offsets, crate labels, and locomotion tolerances as `const` declarations inside the relevant modules.
- [x] Document newly exposed tuning knobs in README/TASK.md to guide future adjustments.
- **Outcome:** Dialogue and economy systems reference descriptive constants/configs instead of ad-hoc literals, reducing duplication and easing tuning.
- **Exit criteria:** No hard-coded literals remain in the touched modules without an accompanying constant or config rationale.

---

## Step S1.11 - Systems Modularisation
**Goal:** Split oversized files so each responsibility stays focused and maintainable.

- [x] Break `economy/systems.rs` into `systems::{spawning, day_prep, task_execution, dialogue}` with a `mod.rs` that re-exports public items.
- [x] Extract the dialogue broker into `broker/mod.rs`, `broker/config.rs`, and `broker/openai.rs` while preserving external APIs.
- [x] Update module documentation to reflect the new layout and ensure compile errors surface unused exports.
- **Outcome:** Economy and dialogue code now live in smaller, purpose-driven files that align with the <400 line guideline.
- **Exit criteria:** Project builds without path changes for external callers and each new module owns a cohesive responsibility.

---

## Step S1.12 - Dead Code Sweep
**Goal:** Remove unused helpers revealed after the modularisation pass.

- [x] Run `cargo clippy -D warnings -- -D dead_code` to surface unused functions, imports, and types.
- [x] Eliminate obsolete helpers and redundant re-exports while keeping telemetry/event wiring intact.
- [x] Update docs and planning artifacts with the cleanup summary and remaining risks.
- **Outcome:** The codebase compiles without dead code allowances, and planning docs explain the remaining follow-ups (UI telemetry, work-order promotion).
- **Exit criteria:** Clippy reports no dead code warnings and documentation reflects the leaner module set.

---

## Step S1.13 - Dialogue Broker Verification & Instrumentation
**Goal:** Prove the OpenAI integration works end-to-end and surface its status in logs/telemetry.

- [x] Emit a clear startup log/telemetry flag indicating live vs. fallback broker mode and expose that state for UI/debug overlays.
- [x] Add a hotkey/console command or debug system that enqueues a sample conversation through the existing `DialogueRequestQueue` to smoke-test API wiring on demand.
- [x] Extend the telemetry log (`logs/dialogue_history.jsonl`) and in-game logging so provider errors, rate limits, and fallbacks are easy to diagnose.
- [x] Refresh docs/planning artifacts (`src/dialogue/README.md`, `docs/tech_notes.md`, `CHANGELOG.md`, `.agent/ai_memory.V.1.yaml`, `.agent/tasks.yaml`, `TASK.md`) with the new verification workflow.
- **Outcome:** Startup logs and telemetry now advertise the broker's live/fallback mode, `broker_status` entries appear in the JSONL history, and pressing `F7` queues a dialogue probe so developers can confirm OpenAI connectivity without editing code.
- **Exit criteria:** Running the debug trigger records a response (or failure) with clear provider status, and documentation explains how to confirm live mode.

---

## Step S1.14 - Conversational Triggers & Prompt Revamp
**Goal:** Broaden NPC conversation hooks and enrich prompt context so dialogue feels grounded.

- [ ] Introduce systems that watch schedule transitions, proximity checks, and trade states to enqueue greetings, banter, and haggling lines while respecting existing rate limits.
- [ ] Expand `DialogueContext` builders to include mood (`NpcMotivation`), recent activities, trade metadata, and relationship hints for richer prompts.
- [ ] Refactor prompt templates/topic hints so responses reference shared world state (e.g., morning greetings, crate-side negotiations, day-end recaps).
- [ ] Update affected docs (`src/dialogue/README.md`, `src/economy/README.md`, `src/npc/README.md`, `docs/tech_notes.md`, planning files) with trigger coverage, tuning knobs, and risks.
- **Outcome:** NPCs initiate varied conversations tied to what they are doing, producing text that references the current day, mood, and trading partners.
- **Exit criteria:** Observing a full day shows multiple trigger types firing with context-rich responses, and documentation outlines configuration/cooldown rules.

---

## Step S1.15 - NPC Needs & Self-Directed Decisions
**Goal:** Let villagers evaluate personal needs and influence their schedules/activities organically.

- [ ] Add an `NpcNeeds` component with hunger, thirst, rest, and social meters driven by configurable decay/recovery curves.
- [ ] Build a decision scoring system that weighs needs, `NpcMotivation`, and schedule commitments to select the next action or dialogue topic.
- [ ] Integrate economy task assignment so villagers can defer work, seek resources, or request help when critical needs fall below thresholds.
- [ ] Expose telemetry/debug overlays for current need levels and chosen actions; document behaviour in module READMEs, tech notes, plan files, and changelog.
- **Outcome:** NPCs occasionally reprioritise tasks or conversations based on their state instead of blindly following scripted loops.
- **Exit criteria:** Simulations demonstrate needs influencing activity choices without deadlocks, and docs explain tuning plus safe defaults.

---

## Step S1.16a - Speech Bubble MVP (Basic Text Above NPCs)
**Goal:** Deliver minimal viable speech bubbles that display dialogue text above NPC heads.

- [x] Create `src/ui/speech_bubble/` module structure with components, systems, and plugin files.
- [x] Define `SpeechBubble` component tracking NPC ID, speaker entity, and lifetime timer.
- [x] Define `SpeechBubbleSettings` resource with configurable lifetime, fade, distance, and font size.
- [x] Define `SpeechBubbleTracker` resource ensuring one bubble per NPC at a time.
- [x] Define `SpeechBubbleUiRoot` resource holding the full-screen UI overlay container.
- [x] Implement spawn system that listens to `DialogueResponseEvent` and creates UI NodeBundle entities.
- [x] Implement update system that positions bubbles via `camera.world_to_viewport()` projection every frame.
- [x] Implement lifetime/fade/despawn system that fades out during final 2 seconds, then despawns.
- [x] Implement distance-based culling (hides bubbles beyond 25 world units).
- [x] Register `SpeechBubblePlugin` in `main.rs` after `DialoguePlugin`.
- [x] Update documentation and planning artifacts.
- **Outcome:** UI-based speech bubbles with dark semi-transparent backgrounds appear above NPC heads, track their 3D positions in screen space, fade out smoothly, and despawn after 10 seconds. Implementation uses `camera.world_to_viewport()` instead of Text2d billboards for reliable positioning.
- **Exit criteria:** Running the game shows dialogue bubbles floating above speaking NPCs that track their movement and fade gracefully.

---

## Step S1.16b - Speech Bubble Visual Polish (Distance Culling & Fade)
**Goal:** Add distance-based culling and smooth fade-out animations for polished appearance.

- [ ] Implement distance-based visibility culling system that hides bubbles beyond `max_distance`.
- [ ] Add fade-out animation system that lerps alpha to 0 during final 2 seconds of lifetime.
- [ ] Fine-tune Y-offset to position bubbles nicely above NPC capsule meshes (test with camera angles).
- [ ] Add word wrapping system to keep lines under 40 characters for readability.
- [ ] Update documentation with distance culling parameters and fade behavior.
- **Outcome:** Bubbles fade naturally with distance (simulating real-world speech audibility) and disappear smoothly.
- **Exit criteria:** Distant NPCs' bubbles are hidden/culled, nearby bubbles fade out gracefully before despawning.

---

## Step S1.16c - Speech Bubble Personality (Volume & Mood Integration)
**Goal:** Integrate NPC personality and mood into speech bubble appearance via font size variations.

- [ ] Implement `SpeechVolume` enum (Whisper/Normal/Loud) with font size multipliers (0.6x / 1.0x / 1.5x).
- [ ] Create volume detection system that analyzes dialogue content (CAPS ratio, whisper keywords).
- [ ] Integrate with `NpcMotivation.mood` to adjust font size (Depressed -20%, Energised +20%).
- [ ] Add configurable max distances per volume level (Whisper 15u, Normal 25u, Loud 40u).
- [ ] Document volume detection keywords and mood modifiers in module README.
- [ ] (Optional) Add per-NPC `Personality` component for persistent volume traits (Boisterous/Shy).
- **Outcome:** NPCs have distinct "voices" through text size - shy/depressed NPCs whisper, energetic NPCs speak louder.
- **Exit criteria:** Font sizes vary based on content and mood; distance culling respects volume levels.

---

## What Comes Next
Use the S1.5 blueprint to draft implementation tasks for Step 7: load profession/recipe configs, add work-order queues, expand economy events, and generate the resource dependency matrix. Follow that by spiking the S1.7 motivation system so wellbeing can feed back into schedules and product quality.






