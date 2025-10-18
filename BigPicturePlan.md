# Big Picture Plan

This plan stitches together the long-term direction (Masterplan), the roadmap milestones, and the concrete steps already delivered. It is intended as a single glance reference for where the project has been and where it is heading.

## Vision Snapshot
We are building a medieval life simulation anchored on a deterministic ECS core, with LLM-augmented dialogue and a data-driven economy. The immediate objective is a believable village slice: villagers follow daily routines, converse about their activities, and trade goods in a loop that survives save/load cycles. This slice sets the foundation for scaling into weather, threats, and multiplayer later.

## Completed Foundations
### Simulation Core (S0 Track)
- **S0.1a  Confirm toolchain & baseline run:** Tooling standardized on Rust 1.90.0 with Bevy 0.17 pre-release; formatting, linting, and `cargo run` verified.
- **S0.1b  CorePlugin & SimulationClock:** Custom SimulationClock resource introduced and registered through `CorePlugin`, giving the project explicit control over time scaling.
- **S0.1c  core_debug feature & logging:** Feature-gated diagnostics provide once-per-second timing logs without polluting release builds; VS Code tasks expose the toggle.
- **S0.2a  World shell bootstrap:** WorldPlugin scaffolds ground plane, lighting, and a fly camera so the scene is explorable.
- **S0.2b  World clock & day/night cycle:** Configuration-driven `WorldClock` ties into SimulationClock, steering sun rotation and ambient lighting.
- **S0.3a  Documentation & automation sweep:** Project docs and planning artifacts aligned around the new world/time systems, closing the S0 milestone.

### NPC & Dialogue Slice (S1 Track to Date)
- **S1.1a  NPC identities & debug spawner:** Villagers spawn with unique identity data via `NpcPlugin` scaffolding.
- **S1.1b  Schedule scaffold & tests:** ScheduleTicker accumulates simulation time before advancing NPC state, with coverage to guard registration.
- **S1.2  Dialogue scaffolding research:** Dialogue API options, rate limiting strategy, and prompt templates documented.
- **S1.3 - Dialogue broker prototype:** DialoguePlugin now offers a queueing broker with rate limiting, structured errors, and response events.
- **S1.4 - Config-driven micro trade planner:** Recipes and daily requests now load from configuration, producing per-profession task queues instead of the hard-coded farmer -> miller -> blacksmith loop.
- **S1.5 - Economy foundation blueprint:** Long-form economy design (resources, work orders, event taxonomy) captured for future implementation.
- **S1.6 - NPC locomotion & profession crates:** Profession crates spawned, locomotion systems move NPCs to crates, and movement telemetry is logged.
- **S1.7 - NPC motivation & wellbeing spike:** Dopamine meters, mood thresholds, alcohol boosts/hangovers, and dependency-driven rewards now influence villager wellbeing once each world day advances.
- **S1.8 - Dialogue telemetry persistence:** Dialogue responses and failures stream to `logs/dialogue_history.jsonl`, keeping offline analysis aligned with in-memory telemetry.
- **S1.9 - Baseline verification & responsibility map:** Toolchain checks re-confirmed the build, and a refreshed responsibility map documents how plugins interact post-refactor.
- **S1.10 - Economy & dialogue literal audit:** OpenAI defaults, trade placeholder offsets, and locomotion tolerances now live in named constants/config files instead of scattered literals.
- **S1.11 - Systems modularisation:** Economy systems split into `spawning`, `day_prep`, `task_execution`, and `dialogue` modules while the dialogue broker moved into `broker/{mod,config,openai}`.
- **S1.12 - Dead code sweep:** Redundant helpers and unused exports removed with `clippy -D dead_code`, keeping the reorganised modules lint-clean.

## Current Position
The economy slice now plans daily work from configuration: villagers wait at their crates, manufacture goods via queued tasks, and deliver them when partners are present. Dialogue telemetry persists to `logs/dialogue_history.jsonl`, and both economy and dialogue modules have been split into focused submodules with shared constants replacing prior magic numbers. Locomotion/economy telemetry has not yet surfaced in UI, and the dependency matrix remains to be generalised for Step 7.

## Upcoming Work (Near-Term)
1. **UI status surfacing:** Feed locomotion, motivation, and planner telemetry into HUD elements or debug overlays, making villager activity legible in real time.
2. **Dialogue broker integration:** Harden the OpenAI path, unify telemetry for UI consumption, and expose configuration validation for the new constants.
3. **Work-order formalisation:** Promote the ad-hoc task queues into Step 7's planned work-order data structures and align the dependency matrix with the config-driven economy tables.

## Roadmap Outlook (Mid to Long Term)
- **M2 - Persistence Layer:** Introduce SQLite-backed save/load, with migrations and world snapshotting so sessions can resume reliably.
- **M3 - NPC Foundations Expansion:** Broaden NPC traits, needs, and spawners to populate the settlement with diverse villagers.
- **M4 - Dialogue Iteration:** Solidify LLM client behavior, prompt templates, rate limits, and chat UI for richer conversations.
- **M5 - Economy Systems:** Implement data-driven resources, job outputs, and market balancing hooks to extend the micro trade loop.
- **M6 - Weather & Seasons:** Layer in weather states and seasonal effects that influence schedules and yields.
- **M7 - Threats & Combat:** Prototype aggro models and combat loops, potentially introducing physics integration.
- **M8 - Lineage:** Track genetics and family trees to enable multi-generational storytelling.
- **M9 - Modding Hooks:** Allow external data packs with validation tooling and plugin discovery.
- **M10 - Multiplayer Prototype:** Explore headless server ticks, client synchronization, and rollback strategies.

## NPC Motivation & Wellbeing Direction
- **Reward resource design:** Dopamine is now a bounded scalar stored per NPC, with configurable gains from tasks, social exchanges, leisure, and daily dependency satisfaction. Future work can extend the config with profession-specific modifiers.
- **Quality vs. self-medication trade-off:** Alcohol already provides a temporary boost followed by a configurable hangover crash and diminished work rewards. Explore additional coping tools (e.g., festivals, comfort food) once more goods exist.
- **Mood states:** Mood descriptors (energised, content, tired, depressed) already influence logging and telemetry; upcoming schedule work should let moods modulate behaviour, dialogue tone, and productivity modifiers.
- **Extensibility:** The motivation module is data-driven via TOML. Additional neurotransmitters or alternative wellbeing models can extend the config/resource without rewriting NPC scheduling.

## Monitoring & Documentation Expectations
Every behavior change must refresh the CHANGELOG, relevant READMEs, tech notes, and the planning artifacts in `.agent/`. Risks, assumptions, and discoveries should be recorded promptly in `ai_memory.V.N.yaml` to keep this big picture accurate.







