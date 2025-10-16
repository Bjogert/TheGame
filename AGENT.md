# AGENT.md - Medieval Simulation (AI-Driven)

This file is the operating manual for any coding assistant that touches the project. Keep it current; everything else (tasks, memory, docs) should align with what is written here.

---

## 0) Baseline & Environment
- **Engine:** Bevy `0.17` (pre-release track; verify crate availability before adding dependencies). Pin the exact patch once the crate stabilises.
- **Language:** Rust 1.78+ (edition 2021). Ensure `rustfmt` and `clippy` components are installed through `rustup`.
- **Target:** Windows desktop first. Linux should work opportunistically; macOS is aspirational.
- **Physics:** Delay until simulation needs it. When required, prefer `bevy_rapier3d` with a compatibility audit against the tracked Bevy version.
- **Style:** Types/traits/enums in `PascalCase`; modules and files in `snake_case`. Commit only code formatted with `cargo fmt` and linted with `cargo clippy -D warnings`.
- **Documentation cadence:** Any behaviour change must update `CHANGELOG.md`, `docs/tech_notes.md`, relevant `/src/**/README.md`, `.agent/tasks.yaml`, and `.agent/ai_memory.V.N.yaml` before the step is considered complete.

---

## 1) Operating Protocol (Meta)
- **Plans drive the work.**
  1. **Plan 1 - Masterplan (this section + Section 2):** Architecture north star. Changes only when direction shifts.
  2. **Plan 2 - Roadmap (Section 3):** Milestones with dependencies. Refresh when scope or order changes.
  3. **Plan 3 - Active Steps (Section 4):** The exact set of tasks currently being executed. Treat them as binding until finished or replaced.
- **Single source of truth:** `.agent/` holds the coordination artifacts. Do not scatter competing plans across comments or ad-hoc docs.
- **Communication:** Capture new assumptions, constraints, or discoveries in `ai_memory.V.N.yaml` under `short_term` (and promote to `long_term` when durable).
- **Refactors:** Allowed when they reduce debt encountered while executing a step. Record rationale and follow-ups in `ai_memory`.
- **Debug tooling:** Add instrumentation under feature flags or config toggles only. Remove or gate it once the need passes.
- **Testing discipline:** When logic becomes complex (economy math, schedules, save/load), add unit/integration tests. If tests are skipped, list the risk explicitly.

---

## 2) Masterplan (Architecture & Principles)
**Vision:** A long-horizon medieval life simulation where autonomous NPCs live multi-generational stories. The player starts in first-person, with eventual multiplayer support once systems mature.

### 2.1 Core Pillars
- **Simulation-first:** Tick-driven ECS with systems communicating through data, events, and well-scoped resources.
- **LLM-augmented NPCs:** Deterministic simulation for needs/schedules; LLMs unlock nuanced dialogue and decision hints when state fidelity exists.
- **Data-driven knobs:** Expose time scaling, economy inputs, and narrative pacing via config files rather than hard-coded constants.
- **Observability:** Provide lightweight metrics/logs (entity counts, tick rate, perf spans) to maintain visibility into the simulation.

### 2.2 Target Module Layout
```
/src
  /core        # App entry, time scaling, logging, profiling toggles
  /world       # Terrain, environmental clocks, spatial queries
  /npc         # Components, behaviour schedulers, spawners
  /dialogue    # LLM client, prompt templates, rate limiting
  /economy     # Resources, jobs, production/consumption loops
  /save        # Serialization, migrations, persistence glue
  /ui          # HUD, menus, chat panels
  /weather     # Weather simulation and environmental effects
  /mods        # Data pack loading (future)
  /utils       # Shared helpers (math, logging, asset loaders)
/config        # Tunables (time.toml, economy.toml, etc.)
/assets        # Art/audio/fonts/placeholders
/docs          # Architecture notes, diagrams, tech decisions
```
Keep files smaller than ~400 lines. Split modules when new responsibilities appear (e.g., `/npc/systems/aging.rs`).

### 2.3 Plugin Dependencies
- **Core plugins (explicit):** `CorePlugin`, `WorldPlugin`, `NpcPlugin`, `EconomyPlugin`, `SavePlugin`.
- **Optional plugins (shared resources):** `DialoguePlugin`, `UiPlugin`, `WeatherPlugin`, `ModsPlugin`.
- **Coordination pattern:** Use events for cross-plugin signalling and read-only resources for shared state. Avoid direct mutable access to external components.

### 2.4 Data Strategy
- **Persistence:** Implement a `DbResource` abstraction once save/load is needed (SQLite-backed). Store migrations in `/migrations` with a version manifest.
- **Configs:** Load TOML configuration at startup. Hot reload is optional but should be designed for.
- **LLM Memory:** Store per-NPC summaries and rolling interaction logs. Bound token usage with summarisation and TTL windows.

### 2.5 Tech Debt Guardrails
- Document trade-offs immediately.
- Track experiments (e.g., physics integration) in `ai_memory.short_term[].experiments`.
- Record issues + fixes in `postmortems` to prevent regression loops.

---

## 3) Roadmap (Plan 2)
| Milestone | Focus | Key Deliverables | Dependencies |
|-----------|-------|------------------|--------------|
| **M0** | Bootstrap & Core | Project skeleton, `CorePlugin` (time scaling, debug toggles), baseline docs | None |
| **M1** | World Slice | Ground plane, lighting, adjustable day/night cycle, player camera | M0 |
| **M2** | Persistence Layer | SQLite wrapper, migrations, save/load of world tick and NPC snapshot | M0 |
| **M3** | NPC Foundations | Identity/traits components, needs & schedule ticks, population spawner | M2 |
| **M4** | Dialogue | LLM client, prompt templates, chat UI, token budgeter | M3 |
| **M5** | Economy | Resource definitions, job outputs, market balancing hooks | M3 |
| **M6** | Weather & Seasons | Weather states affecting schedules and yields | M1, M5 |
| **M7** | Threats & Combat | Aggro models, damage loop, physics integration if required | M1, M3 |
| **M8** | Lineage | Genetics, family trees, trait inheritance | M3 |
| **M9** | Modding Hooks | External data packs, validation tooling, plugin discovery | M5, M8 |
| **M10** | Multiplayer Prototype | Headless server tick, client sync path, rollback exploration | M2-M5 |

**Recurring chores:** Refresh docs/diagrams, audit dependencies, update `ai_memory`, capture postmortems, and remove dead code every milestone.

---

## 4) Active Steps (Plan 3)
> Update this section whenever the top backlog item changes. Keep scope small (0.5-2 focused days).

### Current Focus - S1.3: Dialogue Broker Prototype
- OpenAI stub now guards prompt/ context validation, emits structured errors, and drives logging for dialogue responses/failures.
- DialogueRequestQueue executes with documented cooldown constants and logs rate-limited retries.
- Event consumers bridge TradeCompleted events into dialogue prompts; docs kept in sync with the warning cleanup.

**Recently completed:** S1.2 (dialogue scaffolding research) finished on 2025-10-10; findings documented in docs/dialogue_research.md. S1.6 (profession crates + locomotion) delivered so the micro trade loop now waits for NPCs to reach their work spots.

**Next in queue:** Replace the stub with a real OpenAI client and persist dialogue telemetry for UI hooks. Use the new movement telemetry to surface travel/exchange status in upcoming UI work.

---

## 5) MCP Server Tools
AI assistants working on this project have access to configured Model Context Protocol servers:

### 5.1 Available MCP Servers
- **filesystem** - Navigate and modify project files efficiently
  - Command: `npx @modelcontextprotocol/server-filesystem c:\Users\robert\TheGame`
  - Use for: Reading code, editing files, understanding project structure
- **memory** - Persistent knowledge graph for NPC relationships and world state
  - Command: `npx @modelcontextprotocol/server-memory`
  - Use for: Storing NPC knowledge, tracking relationships, world events
  - Schema: Entities (NPCs, settlements, events), Relations (connections), Observations (facts)
- **serena** - Enhanced conversational AI assistance and task coordination
  - Use for: Intelligent task planning, context-aware assistance, workflow optimization
  - Integration: Complements other MCP servers with advanced reasoning capabilities

### 5.2 Usage Guidelines
- **File Operations:** Use filesystem server for all code reading/writing instead of manual file operations
- **NPC Memory Design:** Plan knowledge graph schema for NPCs, settlements, relationships, and historical events
- **Integration Points:** 
  - DialogueBroker should query memory server for NPC context
  - Save/load systems should sync with memory server
  - World events should create memory entities and relations

---

## 6) Tooling & Automation
- Use VS Code tasks from `.vscode/tasks.json` for routine commands (Run, Check, Clippy, Fmt, Test, Doc, Watch).
- Before merging, run `cargo fmt`, `cargo clippy -D warnings`, and `cargo check --all-targets` at minimum.
- For live-edit loops, prefer `cargo watch -x "check --all-targets"` or `-x run` (install `cargo-watch`).
- Record custom scripts in `/scripts` and link them from `README.md`.

---

## 7) Trusted References
- Bevy Book and example repository matching the tracked minor version.
- Official Bevy migration guides (e.g., 0.16 -> 0.17) once available.
- `bevy_rapier` docs for physics, `bevy_ecs` deep dives for ECS patterns, `leafwing_input_manager` or similar when input abstractions are needed.
- SQLite with Rust: `sqlx` or `rusqlite` docs, plus SQLite official documentation for schema design.

Add better sources (talks, devlogs, whitepapers) here as they surface.










