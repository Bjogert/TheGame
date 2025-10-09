# Changelog

All notable changes to this project will be documented in this file.

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
