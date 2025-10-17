# CLAUDE.md - Medieval Simulation (AI-Driven)

This file mirrors `AGENT.md` so Claude Code follows the same operating protocol when collaborating on **TheGame**.

---

## 0) Baseline & Environment
- **Engine:** Bevy `0.17` (pre-release). Verify crate compatibility before adding dependencies and lock the exact patch once stable.
- **Language:** Rust 1.78+ (edition 2021) with `rustfmt` and `clippy` components installed.
- **Target Platform:** Windows desktop first; Linux should work opportunistically; macOS is aspirational.
- **Physics:** Postpone until required. When needed, prefer `bevy_rapier3d` after confirming Bevy version support.
- **Style:** Types/traits/enums use `PascalCase`; modules/files use `snake_case`. Only commit code after `cargo fmt` and `cargo clippy -D warnings` pass.
- **Documentation cadence:** Any behaviour change must update `CHANGELOG.md`, `docs/tech_notes.md`, relevant `src/**/README.md`, `.agent/tasks.yaml`, and `.agent/ai_memory.V.N.yaml`.

---

## 1) Operating Protocol (Meta)
- **Plans drive the work.**
  1. **Plan 1 – Masterplan:** Architecture north star (Sections 1–2 of `AGENT.md`). Modify only when direction shifts.
  2. **Plan 2 – Roadmap:** Milestones with dependencies (Section 3). Refresh when scope/order changes.
  3. **Plan 3 – Active Steps:** Current execution queue (Section 4). Treat entries as binding until delivered or superseded.
- `.agent/` holds all coordination artifacts. Do not create parallel plans elsewhere.
- Capture new assumptions or discoveries in `.agent/ai_memory.V.N.yaml` (`short_term`, promote durable notes to `long_term`).
- Refactors are welcome when they unblock the current task; record rationale and follow-ups in `ai_memory`.
- Add debug tooling behind feature flags or config toggles; remove or gate once the need passes.
- When logic becomes complex (economy math, scheduling, save/load), add unit/integration tests. If skipping tests, state the risk explicitly.

---

## 2) Masterplan (Architecture & Principles)
**Vision:** Long-horizon medieval life simulation where autonomous NPCs live multi-generational stories. First-person start with eventual multiplayer support once systems mature.

### 2.1 Core Pillars
- Simulation-first ECS ticking via data, events, and scoped resources.
- LLM-augmented NPCs: deterministic needs/schedules, with LLMs supplying rich dialogue and decision hints when state fidelity allows.
- Data-driven knobs: expose time scaling, economy inputs, and narrative pacing via config files rather than constants.
- Observability: lightweight metrics/logs (entity counts, tick rate, perf spans) for simulation visibility.

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
Keep files under ~400 lines; split modules when responsibilities grow.

### 2.3 Plugin Dependencies
- **Core plugins:** `CorePlugin`, `WorldPlugin`, `NpcPlugin`, `EconomyPlugin`, `SavePlugin`.
- **Optional plugins:** `DialoguePlugin`, `UiPlugin`, `WeatherPlugin`, `ModsPlugin`.
- Prefer events for cross-plugin signalling and read-only shared resources. Avoid direct mutable access into external components.

### 2.4 Data Strategy
- Persistence: introduce a `DbResource` abstraction (SQLite-backed) once save/load arrives. Store migrations in `/migrations` with version manifest.
- Configs: load TOML at startup, design for hot-reload compatibility.
- LLM Memory: maintain per-NPC summaries and rolling interaction logs with bounded token usage via summarisation and TTL windows.

### 2.5 Tech-Debt Guardrails
- Document trade-offs immediately.
- Track experiments in `ai_memory.short_term[].experiments`.
- Record issues/fixes in postmortems to prevent regression loops.

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

**Recurring chores:** refresh docs/diagrams, audit dependencies, update `ai_memory`, capture postmortems, remove dead code each milestone.

---

## 4) Active Steps (Plan 3)
- **Current Focus – S1.3: Dialogue Broker Prototype**
  - OpenAI stub guards prompt/context validation, emits structured errors, and logs dialogue responses/failures.
  - `DialogueRequestQueue` executes with documented cooldown constants and logs rate-limited retries.
  - Event consumers bridge `TradeCompleted` events into dialogue prompts; docs synced with warning cleanup.
- **Recently completed:** S1.2 dialogue scaffolding research (2025-10-10) and S1.6 profession crates + locomotion, so NPCs now walk to work spots before trading.
- **Next in queue:** Replace the stub with a real OpenAI client, surface locomotion telemetry in UI, and build the profession/resource dependency matrix for Step 7 so wellbeing hooks and economy configs share a single source of truth.

---

## 5) MCP Server Tools
- **filesystem** – Navigate/modify project files (`npx @modelcontextprotocol/server-filesystem c:\\Users\\robert\\TheGame`).
- **memory** – Persistent knowledge graph for NPC relationships/world state (`npx @modelcontextprotocol/server-memory`).
- **serena** – Enhanced conversational AI/task coordination server.

**Usage guidelines**
- Use filesystem server for file operations instead of manual editing.
- Shape NPC memory schema around entities, relations, and observations.
- `DialogueBroker` should consult the memory server for context.
- Save/load systems must sync with the memory server.
- World events should create memory entities/relations.

---

## 6) Tooling & Automation
- VS Code tasks in `.vscode/tasks.json` cover run, check, clippy, fmt, test, docs, watch.
- Before merging, run `cargo fmt`, `cargo clippy -D warnings`, and `cargo check --all-targets` at minimum.
- For live-edit loops, prefer `cargo watch -x "check --all-targets"` or `-x run` (requires `cargo-watch`).
- Record custom scripts in `/scripts` and link them from `README.md`.

---

## 7) Trusted References
- Bevy Book + examples for the tracked minor version.
- Official Bevy migration guides.
- `bevy_rapier` documentation for physics, `bevy_ecs` deep dives for ECS patterns, `leafwing_input_manager` (or similar) for input abstractions.
- SQLite + Rust docs (`sqlx`, `rusqlite`).
- Add better sources (talks, devlogs, whitepapers) as they surface.

---

## Common Commands & Workflows
```bash
# Run the application
cargo run

# Run with debug features
cargo run --features core_debug

# Format, lint, and type-check (REQUIRED before commits)
cargo fmt
cargo clippy -- -D warnings
cargo check --all-targets

# Run tests
cargo test

# Live reload (requires cargo-watch)
cargo watch -x "check --all-targets"
```

### Maintenance Checklist
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Run `cargo test`
- [ ] Exercise `cargo run` and `cargo run --features core_debug`

### Dependency Guardrails
- Verify Bevy 0.17 compatibility and licensing before adding dependencies.
- Document rationale for new tooling in Section 6.

---

## Quick Reference
- **Entry Point:** `src/main.rs`
- **Time Scaling:** `src/core/plugin.rs`
- **Day/Night Cycle:** `src/world/time.rs`
- **Camera Controls:** `src/world/systems.rs`
- **NPC Systems:** `src/npc/systems.rs`
- **Configuration:** `config/time.toml`

---

## Observability & Debugging
- Camera controls: right-click + drag (mouse look), WASD/Space/LShift for movement, LCtrl for sprint (2.5x).
- NPC debug meshes: capsule visualisation with colour-coded actors.
- Logging levels: `info!` for configuration/state changes, `warn!` for fallbacks/validation issues; prefer feature-gated logging over `debug!`.

---

## Motivation & Economy Outlook
- The profession/resource dependency matrix arrives with S1.7 to describe production/consumption relationships and shared upkeep needs.
- A dopamine-inspired wellbeing system will be prototyped immediately after the matrix: NPCs earn dopamine from task completion and socialisation, can temporarily boost via alcohol, suffer quality penalties while intoxicated, and experience post-binge crashes that risk depressive states if dopamine stays low.
- Keep the model extensible so morale/stress alternatives can replace or augment dopamine if playtests demand.

