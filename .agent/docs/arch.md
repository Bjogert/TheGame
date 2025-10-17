# Architecture Snapshot

This document captures the intended module layout and main dependencies. Update it when architectural direction changes, not for every implementation tweak.

---

## Manifest Baseline (`Cargo.toml` excerpt)
```toml
[package]
name    = "thegame"
version = "0.1.0"
edition = "2021"

[features]
default = []
core_debug = []

[dependencies]
bevy  = "0.17"
serde = { version = "1.0", features = ["derive"] }
toml  = "0.8"
# Add crates opportunistically once Bevy 0.17 compatibility is confirmed:
# bevy_rapier3d = "0.26"        # Physics (check exact compatible release)
# bevy-inspector-egui = "0.23"  # Debug inspector (optional)
# leafwing-input-manager = "0.13" # Input abstraction (optional)
```

Profiles already enable higher optimisation for dependencies. Document extra features (e.g., `dynamic`) in `CHANGELOG.md` when toggled.

---

## Plugin Graph (Current Focus)
```
CorePlugin
  ├─ provides: SimulationClock, DebugFlags
  └─ precedes ──► WorldPlugin

WorldPlugin
  +- depends on: SimulationClock
  +- provides: WorldClock, WorldTimeSettings
  +- systems: advance_world_clock(), apply_world_lighting(), fly camera controls
  +- precedes --? NpcPlugin

NpcPlugin
  +- depends on: WorldPlugin (for world context)
  +- provides: Identity component & NpcIdGenerator
  +- systems: spawn_debug_npcs(), tick_schedule_state(), motivation reward/decay loops

Optional plugins (planned): DialoguePlugin, UiPlugin, WeatherPlugin
```

Guidelines:
- Solid arrow = explicit `add_plugins` ordering.
- Optional plugins should fail gracefully if dependencies are missing (feature flags/config gating).

---

## Data Flow Highlights
- `SimulationClock` (Core) → global delta, time-scale multiplier, tick counters.
- `WorldClock` (World) → time-of-day fraction + day count; drives sun rotation, ambient lighting, and will later feed weather/NPC schedules.
- `/config/time.toml` → parsed once at startup into `WorldTimeSettings`; invalid files fall back to defaults with a warning.
- `PrimarySun` component marks the directional light that responds to the world clock.
- Npc: `Identity` component and `NpcIdGenerator` resource supply unique ids for debug NPCs (expand to registry later).
- Npc Motivation: `MotivationConfig` loads from `config/motivation.toml`, `NpcMotivation` components track dopamine/mood, and `DailyDependencyTracker` consumes economy snapshots to reward or penalise wellbeing.
- Future: `NpcRegistry` for active entities, `DbResource` (Save) for SQLite state, `LlmJobQueue` (Dialogue) for prompt scheduling.

Record concrete resource/component names in the nearest module README once implemented.

---

## Near-Term Decisions
- **Input abstraction:** decide when (or if) to adopt `leafwing_input_manager` before more advanced first-person controls.
- **Asset pipeline:** choose placeholder asset format (primitives vs. lightweight GLTF) for NPC prototyping.
- **Persistence crate:** confirm whether to hand-roll SQLite integration or leverage an ECS snapshot crate.
- **Task scheduling:** evaluate `bevy_tasks` vs. custom async executors for LLM calls once dialogue work begins.

Capture outcomes and rationale in `ai_memory.V.N.yaml` when decisions land.

