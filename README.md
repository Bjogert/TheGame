# TheGame - Medieval Simulation Playground

An experimental Bevy-based project exploring long-form medieval life simulation with AI-assisted NPCs. The repository is intentionally clean: no gameplay yet, just scaffolding ready for systems to plug in.

---

## Getting Started
1. **Install prerequisites**
   - Rust toolchain (stable, 1.78+ recommended)
   - `rustup component add rustfmt clippy`
   - Optional: `cargo install cargo-watch` for live rebuilds
2. **Run the scaffold**
   ```powershell
   cargo run
   ```
   The current binary opens a Bevy window using the default plugins plus our `CorePlugin` and `WorldPlugin`.
3. **Format and lint**
   ```powershell
   cargo fmt
   cargo clippy -- -D warnings
   cargo check --all-targets
   ```
   Every change should pass these before it lands in main.

---

## VS Code Tasks
`.vscode/tasks.json` wires common commands into VS Code's task runner:

- `S0.1a - Toolchain Report` captures `rustup show`.
- `S0.1a - Format / Clippy / Check / Baseline Run` validate the scaffold quickly.
- `S0.1c - Run with core_debug` and `S0.1c - Watch (core_debug)` launch with the debug feature once needed (requires `cargo-watch` for the watch task).
- General utilities (`Run`, `Test`, `Doc`, `Watch (check --all-targets)`) support later milestones.

Run any task via `Ctrl+Shift+P` ? *Run Task*.

---

## Time Configuration
- World time is defined in `config/time.toml`. Adjust `day_length_minutes`, sunrise/sunset fractions, and lighting intensities to customise the cycle.
- Changes require a restart. Invalid files fall back to defaults and emit a warning on startup.

---

## Project Direction
High-level intent and operating rules live in `AGENT.md`. Quick summary:

- **Milestone M0 (current focus):** Core scaffolding, time-scaling, debug toggles, and documentation alignment.
- **Roadmap:** M1 introduces a simple world slice; M2 adds persistence; M3+ expands NPC behaviour, dialogue, economy, weather, and beyond.
- **Active Step:** S0.3a (documentation & automation sweep) as described in `AGENT.md`, `TASK.md`, and `.agent/tasks.yaml`.

Refer to `.agent/ai_memory.V.1.yaml` for decisions, risks, and open questions captured after each step.

---

## Repository Layout
```
TheGame/
+- .agent/                # AI coordination (plans, memory, docs)
¦  +- ai_memory.V.1.yaml
¦  +- tasks.yaml
¦  +- docs/arch.md
+- .vscode/tasks.json     # Editor tasks
+- config/time.toml       # World clock configuration
+- docs/tech_notes.md     # Running technical notes
+- src/
¦  +- core/               # CorePlugin + SimulationClock
¦  +- world/              # WorldPlugin + environment systems
+- npc/                # NpcPlugin + identity/debug spawner
+- TASK.md                # Human-readable step-by-step plan
+- CHANGELOG.md           # Step-level history
+- Cargo.toml             # Tracks Bevy 0.17 + profiles/features
+- README.md              # You are here
```

As systems land, add module-specific README files under `src/<module>/README.md` describing components, resources, and integration points.

---

## Next Actions
Check `.agent/tasks.yaml` for the authoritative backlog. Today the open action is **S0.3a - documentation & automation sweep**. Update the documentation set (README, AGENT, TASK.md, tasks, memory) before promoting the next task.

