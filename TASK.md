# TASK PLAN

_Last updated: 2025-10-08 (UTC). This file explains the step-by-step execution plan so humans can follow the flow easily. For day-to-day coordination between agents, see `.agent/tasks.yaml`._

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
## What Comes Next
Once S0 wraps, move into milestone S1: NPC identity, spawner skeleton, and early dialogue experiments. Those steps will receive their own breakdown once S0.3a turns this list into reality.


