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

### 2.2 Module Layout - Current State

**IMPLEMENTED (as of S1.13):**
```
/src
  /core        # App entry, time scaling (SimulationClock), debug logging [COMPLETE]
  /world       # Terrain, WorldClock, day/night cycle, camera controls [COMPLETE]
  /npc         # Identity, schedules, locomotion, motivation systems [COMPLETE]
    /motivation # Dopamine model, mood states, alcohol mechanics, dependency tracking
  /dialogue    # OpenAI broker, queue, rate limiting, telemetry logging [COMPLETE]
    /broker     # DialogueBroker trait, config, OpenAI live client + fallback
  /economy     # Config-driven planner, recipes, tasks, dependency matrix [COMPLETE]
    /systems    # Spawning, day_prep, task_execution, dialogue triggers
/config        # Tunables: time.toml, motivation.toml, economy.toml [COMPLETE]
/assets        # Art/audio/fonts/placeholders [MINIMAL - basic meshes only]
/docs          # Architecture notes, diagrams, tech decisions [ACTIVE]
```

**PLANNED BUT NOT STARTED:**
```
/src
  /save        # Serialization, migrations, persistence (M2 milestone) [NOT STARTED]
  /ui          # HUD, menus, chat panels, speech bubbles [NOT STARTED]
  /weather     # Weather simulation, environmental effects (M6) [NOT STARTED]
  /mods        # Data pack loading (M9) [NOT STARTED]
  /utils       # Shared helpers (math, logging, asset loaders) [NOT STARTED]
```

Keep files under ~400 lines; split modules when responsibilities grow (enforced in S1.11).

### 2.3 Plugin Dependencies - Actual Registration Order

**Currently Registered (from `main.rs`):**
1. `DefaultPlugins` - Bevy built-in systems
2. `CorePlugin` - SimulationClock, time scaling (provides scaled time to all other plugins)
3. `DialoguePlugin` - OpenAI broker, request queue, rate limiting, telemetry logging
4. `EconomyPlugin` - Recipe registry, task planner, dependency matrix
5. `WorldPlugin` - WorldClock, environment, camera (depends on SimulationClock)
6. `NpcPlugin` - Identity, schedules, locomotion, motivation (depends on WorldPlugin for time context)

**Plugin Coordination:**
- Prefer events for cross-plugin signalling and read-only resources for shared state
- Avoid direct mutable access to external components
- Economy → Dialogue: `TradeCompletedEvent` triggers dialogue requests
- World → Economy: `WorldClock` advances trigger daily task planning
- Economy → NPC Motivation: Trade/dialogue events reward dopamine
- Dialogue responses reward NPC motivation (social interaction bonus)

**Not Yet Implemented:**
- `SavePlugin` (M2 milestone deferred)
- `UiPlugin` (planned for S1.16+)
- `WeatherPlugin` (M6 milestone)
- `ModsPlugin` (M9 milestone)

### 2.4 Data Strategy

**Persistence:**
- **NOT YET IMPLEMENTED.** Milestone M2 (Persistence Layer) deferred below NPC/economy/dialogue foundations.
- When started: introduce `DbResource` abstraction (SQLite-backed) with migrations in `/migrations` and version manifest.
- Current design: all state is in-memory only; NPCs, economy, and world reset on restart.
- No save/load, no data migration systems exist yet.

**Configs (IMPLEMENTED):**
- Load TOML at startup from `/config` directory
- `time.toml` - World clock settings (day length, sunrise/sunset, lighting)
- `motivation.toml` - NPC dopamine model, mood thresholds, alcohol mechanics, dependency bonuses
- `economy.toml` - Recipes, daily requests (production/consumption chains)
- `secrets.env` - Optional environment overrides (OPENAI_API_KEY, model settings) - auto-loaded at startup
- Hot-reload: designed for but not yet implemented (would require file-watching system)

**LLM Memory (PARTIAL):**
- In-memory `DialogueContext` structs with summaries and event histories
- Per-request context limited to trade/schedule events
- Telemetry logged to `logs/dialogue_history.jsonl` (JSONL format for offline analysis)
- No persistent cross-session NPC memory yet
- MCP memory server integration planned but not wired (see Section 5)

### 2.5 Tech-Debt Guardrails
- Document trade-offs immediately.
- Track experiments in `ai_memory.short_term[].experiments`.
- Record issues/fixes in postmortems to prevent regression loops.

### 2.6 Implemented Features Summary (as of S1.13)

**Core Systems:**
- ✅ Time scaling (SimulationClock with configurable multiplier, clamped to MIN_TIME_SCALE)
- ✅ Day/night cycle (WorldClock drives sun rotation and ambient lighting via `time.toml`)
- ✅ Fly camera controls (WASD/Space/LShift movement, right-click mouse look, LCtrl sprint)
- ✅ Feature-gated debug logging (`core_debug` feature for per-second tick telemetry)

**NPC Systems:**
- ✅ Identity component (unique IDs like "NPC-0042", display names, ages)
- ✅ Daily schedules (activity transitions based on WorldClock time-of-day fractions)
- ✅ Locomotion (straight-line ground movement, destination tracking, arrival logging)
- ✅ Motivation system (dopamine model: 0-100 range, decay, rewards from tasks/social/leisure)
- ✅ Mood states (Energised ≥80, Content ≥55, Tired ≥30, Depressed <30 with configurable thresholds)
- ✅ Alcohol mechanics (boost +12, intoxication 90s, hangover -15 penalty, quality penalty 20%)
- ✅ Dependency tracking (daily bonuses +4 for Food+Tools access, penalties -7.5 per missing category)
- ✅ Schedule ticker (5-second cadence, activity transitions logged)

**Dialogue Systems:**
- ✅ OpenAI integration (live API client using `gpt-4o-mini` via `reqwest::blocking::Client`)
- ✅ Fallback mode (activates when OPENAI_API_KEY missing, fabricates responses from context)
- ✅ Request queue with rate limiting (global 1.5s + per-NPC 8s cooldowns, max 2 retries)
- ✅ Context validation (trade topics require trade events, schedule topics require schedule updates)
- ✅ Telemetry logging (`logs/dialogue_history.jsonl` in JSONL format with broker status, responses, failures)
- ✅ Broker status instrumentation (live/fallback mode indicator logged at startup)
- ✅ Debug probe (F7 hotkey enqueues test request for credential verification)
- ✅ Conversational triggers (partial: trade delivery + schedule updates; greetings/status/haggling pending S1.14)
- ✅ Rate limit handling (parses `Retry-After` headers, emits structured `DialogueError`)

**Economy Systems:**
- ✅ Config-driven recipes (`economy.toml` with TOML schema: id, actor, produces, consumes)
- ✅ Task planner (generates WaitForGood/Manufacture/Deliver queues from daily requests)
- ✅ Inventory management (per-NPC HashMap<TradeGood, u32>, synced with visual placeholders)
- ✅ Profession/resource dependency matrix (DependencyCategory: Food/Tools, used by motivation bonuses)
- ✅ Trade events (TradeCompletedEvent with reasons: Production/Processing/Exchange, logged)
- ✅ Profession crates (visible colored cube entities spawned for Farmer/Miller/Blacksmith)
- ✅ Trade good placeholders (spheres spawned on crates based on inventory: Grain/Flour/Tools)
- ✅ Locomotion integration (NPCs walk to crates before executing tasks, visible movement)
- ✅ Dialogue triggers (delivery tasks enqueue dialogue requests with trade context)

**Configuration:**
- ✅ `config/time.toml` (day_length_minutes, sunrise/sunset fractions, sun_declination, lighting lux/colors)
- ✅ `config/motivation.toml` (dopamine min/max/start, decay, gains, thresholds, alcohol, leisure keywords)
- ✅ `config/economy.toml` (recipes array, daily_requests array)
- ✅ `secrets.env` (OPENAI_API_KEY, OPENAI_MODEL, OPENAI_TIMEOUT_SECS, etc. - optional overrides)
- ✅ Environment variable support (OpenAI base URL, temperature, max tokens customizable)

**NOT YET IMPLEMENTED:**
- ❌ Persistence (SQLite, save/load, migrations - M2 milestone deferred)
- ❌ UI plugin (HUD, menus, speech bubbles, telemetry displays - needed for S1.16)
- ❌ Weather systems (M6 milestone)
- ❌ Threats & combat (M7 milestone)
- ❌ Lineage & genetics (M8 milestone)
- ❌ Modding hooks (M9 milestone)
- ❌ Multiplayer (M10 milestone)
- ❌ Pathfinding (straight-line locomotion only, no obstacle avoidance)
- ❌ MCP memory server integration (documented but not wired)

---

## 3) Roadmap (Plan 2) - Updated with Status

| Milestone | Focus | Key Deliverables | Dependencies | **Status** |
|-----------|-------|------------------|--------------|------------|
| **M0** | Bootstrap & Core | Project skeleton, `CorePlugin` (time scaling, debug toggles), baseline docs | None | ✅ **COMPLETE** |
| **M1** | World Slice | Ground plane, lighting, adjustable day/night cycle, player camera | M0 | ✅ **COMPLETE** |
| **M2** | Persistence Layer | SQLite wrapper, migrations, save/load of world tick and NPC snapshot | M0 | ⏸️ **DEFERRED** (prioritized below M3-M5) |
| **M3** | NPC Foundations | Identity/traits components, needs & schedule ticks, population spawner | ~~M2~~ | ✅ **COMPLETE** (motivation system, locomotion, schedules) |
| **M4** | Dialogue | LLM client, prompt templates, ~~chat UI~~, token budgeter | M3 | ✅ **COMPLETE** (OpenAI live client, rate limiting, telemetry; UI pending) |
| **M5** | Economy | Resource definitions, job outputs, ~~market balancing hooks~~ | M3 | 🟡 **PARTIAL** (config-driven recipes, task planner; market balancing pending) |
| **M6** | Weather & Seasons | Weather states affecting schedules and yields | M1, M5 | ⏸️ **NOT STARTED** |
| **M7** | Threats & Combat | Aggro models, damage loop, physics integration if required | M1, M3 | ⏸️ **NOT STARTED** |
| **M8** | Lineage | Genetics, family trees, trait inheritance | M3 | ⏸️ **NOT STARTED** |
| **M9** | Modding Hooks | External data packs, validation tooling, plugin discovery | M5, M8 | ⏸️ **NOT STARTED** |
| **M10** | Multiplayer Prototype | Headless server tick, client sync path, rollback exploration | M2-M5 | ⏸️ **NOT STARTED** |

**Note on M2 Deferral:** Persistence was originally a dependency for M3, but development proceeded without it to prioritize gameplay foundations. All current state is in-memory only.

**Recurring chores:** refresh docs/diagrams, audit dependencies, update `ai_memory`, capture postmortems, remove dead code each milestone (enforced via S1.9-S1.12 cleanup series).

---

## 4) Active Steps (Plan 3)

- **Current Focus – S1.14: Conversational Triggers & Prompt Revamp**
  - Add event-driven dialogue triggers for greetings, status check-ins, and trade haggling grounded in proximity/schedule cues.
  - Expand `DialogueContext` builders with mood, recent activities, and trade metadata to enrich prompts.
  - Refine prompt templates and topic hints so responses reference shared world state.
  - **Partial implementation:** Trade delivery and schedule update triggers exist (from S1.4); greetings/status/haggling remain.
  - Keep rate limits in mind by batching dialog opportunities and reusing cooldown infrastructure.

- **Recently completed:**
  - **S1.13:** Dialogue broker verification & instrumentation (2025-10-19)
    - `DialogueBrokerStatus` resource tracks live vs. fallback mode, logged at startup
    - F7 debug probe hotkey enqueues test request for credential verification
    - Broker status snapshots recorded in `logs/dialogue_history.jsonl`
    - Automatic `secrets.env` loading for API keys
  - **S1.12:** Dead code sweep with `clippy -D dead_code`
  - **S1.11:** Systems modularisation (economy split into spawning/day_prep/task_execution/dialogue; broker split into mod/config/openai)
  - **S1.10:** Economy & dialogue literal audit (OpenAI defaults, trade offsets, locomotion tolerances centralized)
  - **S1.9:** Baseline verification & responsibility map documentation
  - **S1.8:** Dialogue telemetry persistence (`logs/dialogue_history.jsonl` in JSONL format)
  - **S1.7:** NPC motivation & wellbeing spike (dopamine model, alcohol mechanics, dependency matrix)
  - **S1.6:** NPC locomotion & profession crates (NPCs walk to crates before trading)
  - **S1.4-S1.5:** Config-driven economy planner with recipes/daily requests from `economy.toml`

- **Next in queue:**
  - Complete S1.14 conversational triggers (greetings, status checks, haggling)
  - **S1.15:** NPC needs & self-directed decisions (introduce NpcNeeds component tracking hunger/thirst/rest/social, decision system weighing needs vs. motivation)
  - **S1.16:** In-world dialogue bubbles (requires UI plugin foundation, render floating speech bubbles above NPCs)

---

## 5) MCP Server Tools

**Available MCP Servers:**
- **filesystem** – Navigate/modify project files (`npx @modelcontextprotocol/server-filesystem c:\\Users\\robert\\TheGame`).
- **memory** – Persistent knowledge graph for NPC relationships/world state (`npx @modelcontextprotocol/server-memory`).
- **serena** – Enhanced conversational AI/task coordination server.

**Integration Status: DOCUMENTED BUT NOT YET WIRED**
- MCP tools are configured for Claude Code but not integrated into game systems
- `DialogueBroker` currently uses in-memory `DialogueContext` structs without consulting the memory server
- No persistence layer exists to sync with memory server (M2 deferred)
- Filesystem server usage limited to development/documentation tasks

**Planned Integration (Future Work):**
- Shape NPC memory schema around entities (NPCs, settlements, events), relations (family, trade, social), and observations (facts, conversations)
- `DialogueBroker` should query memory server for NPC context before generating prompts
- Save/load systems (M2) must sync with memory server for cross-session persistence
- World events should create memory entities/relations (birth, death, trade, conflict)
- Economy dependency matrix could feed into memory graph for relationship tracking

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

**Key File Locations:**
- **Entry Point:** [src/main.rs](src/main.rs) - Plugin registration, secrets.env loading
- **Time Scaling:** [src/core/plugin.rs](src/core/plugin.rs) - SimulationClock implementation
- **Day/Night Cycle:** [src/world/time.rs](src/world/time.rs) - WorldClock implementation
- **Camera Controls:** [src/world/systems.rs](src/world/systems.rs) - Fly camera movement
- **NPC Systems:** [src/npc/systems.rs](src/npc/systems.rs) - Spawning, scheduling, locomotion
- **NPC Motivation:** [src/npc/motivation/systems.rs](src/npc/motivation/systems.rs) - Dopamine rewards/decay
- **Dialogue Broker:** [src/dialogue/broker/openai.rs](src/dialogue/broker/openai.rs) - OpenAI client implementation
- **Economy Planner:** [src/economy/systems/day_prep.rs](src/economy/systems/day_prep.rs) - Daily task generation

**Configuration Files:**
- [config/time.toml](config/time.toml) - World clock, day/night cycle
- [config/motivation.toml](config/motivation.toml) - NPC dopamine, mood, alcohol
- [config/economy.toml](config/economy.toml) - Recipes, daily requests
- `secrets.env` - OpenAI API key and model overrides (create manually, not in repo)

**Output Files:**
- [logs/dialogue_history.jsonl](logs/dialogue_history.jsonl) - Dialogue telemetry (JSONL format)

---

## Observability & Debugging

**Camera Controls:**
- **Right-click + drag:** Mouse look (cursor grabbed while held)
- **WASD:** Move camera horizontally
- **Space:** Move up
- **LShift:** Move down
- **LCtrl (hold):** Sprint (2.5x speed multiplier)

**Debug Hotkeys:**
- **F7:** Enqueue dialogue probe (tests OpenAI credentials with canned request using first NPC)

**Visual Debugging:**
- NPC capsule meshes with unique colors per identity
- Profession crates (colored cubes: Farmer, Miller, Blacksmith)
- Trade good placeholders (spheres on crates: Grain, Flour, Tools)
- Sun rotation and ambient lighting track world time-of-day

**Logging Levels:**
- `info!` - Configuration loads, state changes, NPC activity transitions
- `warn!` - Fallbacks (missing config/API key), validation issues, hangover crashes
- Feature-gated logging: Run with `--features core_debug` for per-second sim tick telemetry

**Telemetry Files:**
- `logs/dialogue_history.jsonl` - JSONL entries for broker status, responses, failures
- Check broker status on startup: "live mode" vs "fallback mode" indicates OpenAI connectivity

---

## Motivation & Economy - Implementation Details

**Dependency Matrix (IMPLEMENTED in S1.7):**
- `EconomyDependencyMatrix` describes production/consumption relationships and shared upkeep needs
- `DependencyCategory` enum: Food (satisfied by Grain/Flour), Tools (satisfied by Tools trade good)
- Profession requirements: Farmers need Food+Tools, Millers need Food+Tools, Blacksmiths need Food+Tools
- Daily snapshots via `DailyDependencyTracker` queue satisfaction checks until world day advances
- Satisfaction bonuses (+4 dopamine) and deficit penalties (-7.5 per missing category) applied via `evaluate_dependency_impacts`

**Dopamine Wellbeing System (IMPLEMENTED in S1.7):**
- NPCs earn dopamine from task completion (+8), social interaction (+6), and leisure (+5)
- Dopamine range: 0-100 (configurable in `config/motivation.toml`)
- Decay: 0.25/sec by default (adjustable)
- Mood states: Energised (≥80), Content (≥55), Tired (≥30), Depressed (<30)
- All thresholds, gains, and decay rates loaded from TOML config

**Alcohol Mechanics (IMPLEMENTED in S1.7):**
- Trigger keywords: tavern, ale, mead, wine (configurable in `motivation.toml`)
- Immediate boost: +12 dopamine
- Intoxication: 90-second duration with 20% quality penalty on task rewards
- Hangover: -15 dopamine penalty, 1.6x decay multiplier, 180-second duration
- Post-binge crashes logged as warnings when dopamine drops below Tired threshold

**Extensibility:**
- All parameters config-driven via TOML (no hardcoded constants)
- Profession-specific modifiers can be added to `motivation.toml` in future
- Additional neurotransmitters or wellbeing models can extend the config without rewriting NPC scheduling
- Mood states already influence logging/telemetry; future work can modulate dialogue tone and productivity

