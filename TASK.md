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
- **Outcome:** `WorldPlugin` now spawns a 200×200 plane, directional light, and a fly camera with WASD + Space/LShift movement and right-mouse look (cursor grab toggled automatically). Documentation covers usage and future follow-ups.
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

## Step S1.4 - Micro Trade Loop Spike
**Goal:** Prove that simulation events (placeholder trades) can update inventories and feed the dialogue system.

- [x] Introduce an `EconomyPlugin` assigning farmer, miller, and blacksmith professions with inventories.
- [x] Run a daily micro loop creating grain → flour → tool crates, emitting `TradeCompletedEvent` records.
- [x] Queue dialogue requests from trade events to validate the broker context path.
- **Outcome:** Each in-game day now generates trade events that update inventories and trigger contextual dialogue.
- **Exit criteria:** Logs show the trade chain, inventories mutate correctly, and dialogue responses cite the exchange.

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

- [ ] Add a per-NPC dopamine resource with configurable caps, baseline decay, and gains tied to completed tasks, social interactions, and leisure events.
- [ ] Map dopamine thresholds to mood states (content, tired, depressed) that influence schedule modifiers, dialogue tone, and production efficiency.
- [ ] Model alcohol as a temporary dopamine boost with intoxication penalties (reduced product quality) and a hangover crash that dips below the starting point.
- [ ] Connect the motivation data to the economy dependency matrix so resource access (food, tools, housing) can influence long-term wellbeing.
- [ ] Update planning docs, tech notes, and telemetry plans to cover the new resource and its tuning knobs.
- **Exit criteria:** Motivation metrics surface in telemetry/docs, mood thresholds and alcohol side effects are documented, and follow-up tasks for full integration are queued.

---

## What Comes Next
Use the S1.5 blueprint to draft implementation tasks for Step 7: load profession/recipe configs, add work-order queues, expand economy events, and generate the resource dependency matrix. Follow that by spiking the S1.7 motivation system so wellbeing can feed back into schedules and product quality.






