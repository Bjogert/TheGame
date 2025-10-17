# Big Picture Plan

This plan stitches together the long-term direction (Masterplan), the roadmap milestones, and the concrete steps already delivered. It is intended as a single glance reference for where the project has been and where it is heading.

## Vision Snapshot
We are building a medieval life simulation anchored on a deterministic ECS core, with LLM-augmented dialogue and a data-driven economy. The immediate objective is a believable village slice: villagers follow daily routines, converse about their activities, and trade goods in a loop that survives save/load cycles. This slice sets the foundation for scaling into weather, threats, and multiplayer later.

## Completed Foundations
### Simulation Core (S0 Track)
- **S0.1a – Confirm toolchain & baseline run:** Tooling standardized on Rust 1.90.0 with Bevy 0.17 pre-release; formatting, linting, and `cargo run` verified.
- **S0.1b – CorePlugin & SimulationClock:** Custom SimulationClock resource introduced and registered through `CorePlugin`, giving the project explicit control over time scaling.
- **S0.1c – core_debug feature & logging:** Feature-gated diagnostics provide once-per-second timing logs without polluting release builds; VS Code tasks expose the toggle.
- **S0.2a – World shell bootstrap:** WorldPlugin scaffolds ground plane, lighting, and a fly camera so the scene is explorable.
- **S0.2b – World clock & day/night cycle:** Configuration-driven `WorldClock` ties into SimulationClock, steering sun rotation and ambient lighting.
- **S0.3a – Documentation & automation sweep:** Project docs and planning artifacts aligned around the new world/time systems, closing the S0 milestone.

### NPC & Dialogue Slice (S1 Track to Date)
- **S1.1a – NPC identities & debug spawner:** Villagers spawn with unique identity data via `NpcPlugin` scaffolding.
- **S1.1b – Schedule scaffold & tests:** ScheduleTicker accumulates simulation time before advancing NPC state, with coverage to guard registration.
- **S1.2 – Dialogue scaffolding research:** Dialogue API options, rate limiting strategy, and prompt templates documented.
- **S1.3 – Dialogue broker prototype:** DialoguePlugin now offers a queueing broker with rate limiting, structured errors, and response events.
- **S1.4 – Micro trade loop spike:** Farmer → Miller → Blacksmith loop runs daily, emitting trade events that feed dialogue prompts.
- **S1.5 – Economy foundation blueprint:** Long-form economy design (resources, work orders, event taxonomy) captured for future implementation.
- **S1.6 – NPC locomotion & profession crates:** Profession crates spawned, locomotion systems move NPCs to crates, and movement telemetry is logged.
- **S1.7 – NPC motivation & wellbeing spike:** Dopamine meters, mood thresholds, alcohol boosts/hangovers, and dependency-driven rewards now influence villager wellbeing via the new motivation systems once each world day advances.

## Current Position
All S1 tasks through the OpenAI integration are complete. Dialogue telemetry now persists to `logs/dialogue_history.jsonl`, the dialogue broker calls the live OpenAI client when credentials are present (falling back to the deterministic stub otherwise), locomotion telemetry has not yet surfaced in UI, and the dependency matrix remains a placeholder awaiting Step 7's data-driven configs.

## Upcoming Work (Near-Term)
1. **UI status surfacing:** Feed locomotion, motivation, and trade telemetry into HUD elements or debug overlays so villager activity stays legible in real time.
2. **Resource dependency matrix:** Promote the placeholder matrix into Step 7's config-driven tables so economy balancing and wellbeing hooks share a single source of truth.
3. **Dialogue observability polish:** Add log retention/rotation options and a privacy toggle for persisted dialogue once UI surfaces arrive.

## Roadmap Outlook (Mid to Long Term)
- **M2 – Persistence Layer:** Introduce SQLite-backed save/load, with migrations and world snapshotting so sessions can resume reliably.
- **M3 – NPC Foundations Expansion:** Broaden NPC traits, needs, and spawners to populate the settlement with diverse villagers.
- **M4 – Dialogue Iteration:** Solidify LLM client behavior, prompt templates, rate limits, and chat UI for richer conversations.
- **M5 – Economy Systems:** Implement data-driven resources, job outputs, and market balancing hooks to extend the micro trade loop.
- **M6 – Weather & Seasons:** Layer in weather states and seasonal effects that influence schedules and yields.
- **M7 – Threats & Combat:** Prototype aggro models and combat loops, potentially introducing physics integration.
- **M8 – Lineage:** Track genetics and family trees to enable multi-generational storytelling.
- **M9 – Modding Hooks:** Allow external data packs with validation tooling and plugin discovery.
- **M10 – Multiplayer Prototype:** Explore headless server ticks, client synchronization, and rollback strategies.

## NPC Motivation & Wellbeing Direction
- **Reward resource design:** Dopamine is now a bounded scalar stored per NPC, with configurable gains from tasks, social exchanges, leisure, and daily dependency satisfaction. Future work can extend the config with profession-specific modifiers.
- **Quality vs. self-medication trade-off:** Alcohol already provides a temporary boost followed by a configurable hangover crash and diminished work rewards. Explore additional coping tools (e.g., festivals, comfort food) once more goods exist.
- **Mood states:** Mood descriptors (energised, content, tired, depressed) already influence logging and telemetry; upcoming schedule work should let moods modulate behaviour, dialogue tone, and productivity modifiers.
- **Extensibility:** The motivation module is data-driven via TOML. Additional neurotransmitters or alternative wellbeing models can extend the config/resource without rewriting NPC scheduling.

## Monitoring & Documentation Expectations
Every behavior change must refresh the CHANGELOG, relevant READMEs, tech notes, and the planning artifacts in `.agent/`. Risks, assumptions, and discoveries should be recorded promptly in `ai_memory.V.N.yaml` to keep this big picture accurate.
