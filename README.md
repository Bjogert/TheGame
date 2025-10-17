# TheGame - Medieval Simulation Playground

An experimental Bevy-based project exploring long-form medieval life simulation with AI-assisted NPCs. The repository is intentionally clean: no gameplay yet, just scaffolding ready for systems to plug in.

---

## Getting Started
1. **Install prerequisites**
   - Rust toolchain (stable, 1.78+ recommended)
## Docker
- **Heads-up on this repo's hosted workspace:** the execution sandbox used for
  automated agents (including the one that produced this README) does not expose
  the Docker daemon. Attempts to run `docker build` or `docker compose` there
  will fail with "Cannot connect to the Docker daemon"-style errors because the
  container is itself unprivileged. When working on your own machine—with
  Docker Desktop, Rancher, or a native Docker Engine installation—these
  commands succeed normally.
- **Build a release image** (multi-stage Dockerfile):
  `bash
  docker build -t thegame:latest .
  `
  Run the image with GPU/display access: `bash
  docker run --rm \
    --env DISPLAY=$DISPLAY \
    --volume /tmp/.X11-unix:/tmp/.X11-unix \
    --device /dev/dri \
    thegame:latest
  `
  Adjust the display flags for your platform (WSL2 users can forward the display to Windows via an X server such as VcXsrv).
- **Iterate inside a container** using docker compose. The `thegame` service mounts the repository and caches cargo artifacts:
  `bash
  docker compose run --rm thegame cargo check --all-targets
  docker compose run --rm thegame cargo run
  `
  Compose uses the `dev` stage of the Dockerfile, so additional devices (e.g., `/dev/dri`) can be passed via `docker compose run` when you need graphical output.
- **Linux desktop usage:** the container now ships Vulkan userspace drivers. To launch with host rendering:
  1. Determine the render group ID (usually `getent group render` → `109`). Export it as `RENDER_GROUP` when invoking compose if it differs.
  2. Permit the container to talk to your compositor: `xhost +local:` for X11, or share the wayland runtime directory via `XDG_RUNTIME_DIR`.
  3. Run the service with the Linux override: `docker compose -f docker-compose.yml -f docker-compose.linux.yml run --rm thegame cargo run`.
  The override binds `/dev/dri`, forwards the display sockets, and relaxes seccomp enough for wgpu to initialise Vulkan.

---

- Upcoming work will add an EconomyRegistry, WorkOrderQueue, expanded event taxonomy, and a profession/resource dependency matrix so new trades stay readable.
- **Milestone S1 (current focus):** NPC scaffolding, dialogue groundwork, locomotion polish, and economy planning.
- **Active queue:** Expose locomotion/trade status in the UI and publish the expanded dependency matrix for Step 7 now that dialogue telemetry persists to disk.
- Dialogue telemetry now streams to `logs/dialogue_history.jsonl` as JSON lines for offline analysis or external tooling.
- Dialogue integration is currently in the research phase. See docs/dialogue_research.md for provider comparisons and rate-limiting notes captured during Step S1.2.
- The active broker queues requests with global/per-NPC cooldowns while we prepare to swap in real providers.

## NPC Motivation & Wellbeing
- Villager motivation now tracks a dopamine-style meter via `NpcMotivation`, rewarding productive tasks, dialogue responses, and leisure activities while decaying over time.
- Alcohol triggers a configurable boost from `config/motivation.toml`, followed by a hangover crash that amplifies decay and reduces work rewards until the penalty clears.
- Daily wellbeing is tied to the economy dependency matrix: matching goods delivered by professions mark needs as satisfied once the next day begins, while shortages apply penalties that visibly shift mood states.
- Tune dopamine caps, gains, thresholds, and alcohol behaviour by editing `config/motivation.toml`; changes require a restart today.
- **Milestone S1 (current focus):** NPC scaffolding, dialogue groundwork, locomotion, economy planning, and the newly landed motivation slice.
- **Active queue:** Surface locomotion/trade telemetry in the UI and expand the profession dependency matrix for Step 7.
Check .agent/tasks.yaml for the authoritative backlog. Upcoming steps focus on UI telemetry surfacing and promoting the dependency matrix into Step 7's config-driven data. Keep documentation (README, AGENT, TASK.md, tasks, memory) in sync as each slice lands.
3. **Format and lint**
   `powershell
- Dialogue integration is in place at the prototype level. See docs/dialogue_research.md for provider comparisons and rate-limiting notes captured during Step S1.2.
- The active broker queues requests with global/per-NPC cooldowns while we prepare to swap in real providers.

## Economy Blueprint
- Step S1.5 produced docs/economy_blueprint.md outlining how the placeholder micro trade loop evolves into a config-driven economy slice.
- Upcoming work will add an EconomyRegistry, WorkOrderQueue, and expanded event taxonomy feeding dialogue and UI systems.
- **Milestone S1 (current focus):** NPC scaffolding, dialogue groundwork, and economy planning.
- **Active Step:** S1.5 (economy foundation blueprint) completed; Step 7 implementation tasks are now being drafted in the planning artifacts.
│  ├─ dialogue_research.md# Dialogue provider and prompt considerations
│  └─ economy_blueprint.md# Step S1.5 economy design plan

---

## VS Code Tasks
.vscode/tasks.json wires common commands into VS Code's task runner:

- **Milestone S1 (current focus):** NPC scaffolding, dialogue groundwork, trade loop, locomotion polish, and motivation systems.
- **Active focus:** Preparing to integrate the production OpenAI client, surface telemetry in the UI, and promote the dependency matrix into Step 7's config-driven data.
- S0.1a - Format / Clippy / Check / Baseline Run validates the scaffold quickly.
- S0.1c - Run with core_debug and S0.1c - Watch (core_debug) launch with the debug feature once needed (requires cargo-watch).
- General utilities (Run, Test, Doc, Watch (check --all-targets)) support later milestones.

Run any task via Ctrl+Shift+P → *Run Task*.

---

## Time Configuration
- World time is defined in config/time.toml. Adjust day_length_minutes, sunrise/sunset fractions, and lighting intensities to customise the cycle.
- NPC schedules currently tick every ~5 seconds of simulation time; expect capsule villagers to change activities periodically in the logs.
- Changes require a restart. Invalid files fall back to defaults and emit a warning on startup.

## Dialogue Research
- Dialogue integration is currently in the research phase. See docs/dialogue_research.md for provider comparisons and rate-limiting notes.
- Plan: start with managed LLM APIs (OpenAI), wrap requests behind a DialogueBroker abstraction, and enforce both global and per-NPC rate limits with queued requests for backpressure.

---

## Project Direction
High-level intent and operating rules live in AGENT.md. Quick summary:

- **Milestone S1 (current focus):** NPC scaffolding and dialogue research.
- **Roadmap:** M1 introduces a simple world slice; M2 adds persistence; M3+ expands NPC behaviour, dialogue, economy, weather, and beyond.
- **Active Step:** S1.2 (dialogue scaffolding research) as described in AGENT.md, TASK.md, and .agent/tasks.yaml.

Refer to .agent/ai_memory.V.1.yaml for decisions, risks, and open questions captured after each step.

---

## Repository Layout
`
TheGame/
├─ .agent/                # AI coordination (plans, memory, docs)
│  ├─ ai_memory.V.1.yaml
│  ├─ tasks.yaml
│  └─ docs/arch.md
├─ .vscode/tasks.json     # Editor tasks
├─ config/time.toml       # World clock configuration
├─ docs/
│  ├─ tech_notes.md       # Running technical notes
│  └─ dialogue_research.md# Dialogue provider and prompt considerations
├─ src/
│  ├─ core/               # CorePlugin + SimulationClock
│  ├─ world/              # WorldPlugin + environment systems
│  └─ npc/                # NpcPlugin + identity/debug spawner
├─ TASK.md                # Human-readable step-by-step plan
├─ CHANGELOG.md           # Step-level history
├─ Cargo.toml             # Tracks Bevy 0.17 + profiles/features
└─ README.md              # You are here
`

As systems land, add module-specific README files under src/<module>/README.md describing components, resources, and integration points.

---

## Next Actions
Check .agent/tasks.yaml for the authoritative backlog. Today the open action is **S1.2 - dialogue scaffolding research**. Update the documentation set (README, AGENT, TASK.md, tasks, memory) before promoting the next task.
