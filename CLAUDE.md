# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**TheGame** is a medieval life simulation built with Bevy 0.17, exploring long-form NPC simulation with AI assistance. The codebase (865 lines of Rust) demonstrates professional engineering patterns with clean plugin-based ECS architecture and data-driven configuration.

**Current State:** Milestone S1.1b (schedule scaffold active)
**Architecture Maturity:** Early but well-designed; ready for horizontal expansion
**Code Quality:** 61% documentation-to-code ratio, defensive validation throughout, no files exceed 400-line guideline

**Key Architectural Strengths:**
- Clean 3-tier plugin dependency graph (Core → World → NPC)
- Comprehensive time-scaling system cascading through all simulation layers
- Defensive programming with validation, clamping, and fallback patterns
- Feature-gated debug instrumentation with zero runtime cost
- No circular dependencies detected

## Common Commands

### Development Workflow
```powershell
# Run the application
cargo run

# Run with debug features (per-second simulation tick logging)
cargo run --features core_debug

# Format, lint, and check before commits (REQUIRED)
cargo fmt
cargo clippy -- -D warnings
cargo check --all-targets

# Run tests
cargo test

# Live reload during development (requires cargo-watch)
cargo watch -x "check --all-targets"
cargo watch -x "run --features core_debug"
```

### VS Code Tasks
Pre-configured tasks accessible via `Ctrl+Shift+P` → *Run Task*:
- `S0.1a - Format / Clippy / Check / Baseline Run` - Full validation pipeline
- `S0.1c - Run with core_debug` - Launch with debug logging
- `General - Test (cargo test)` - Run test suite
- `General - Watch (check --all-targets)` - Continuous type-checking

## Architecture Deep Dive

### Plugin Dependency Graph

**CRITICAL: Plugin registration order is mandatory and enforced:**

```
CorePlugin (Foundation)
    └── provides: SimulationClock
        │
        ├── WorldPlugin (Environment)
        │   ├── depends on: SimulationClock
        │   └── provides: WorldClock, WorldTimeSettings, PrimarySun
        │       │
        │       └── NpcPlugin (Entities)
        │           ├── depends on: WorldClock (via schedule updates)
        │           └── provides: Identity, NpcIdGenerator, DailySchedule
```

**Registration in [src/main.rs:11-16](src/main.rs#L11-L16):**
```rust
App::new()
    .add_plugins((
        DefaultPlugins,
        CorePlugin::default(),  // MUST be first - provides SimulationClock
        WorldPlugin,            // Depends on CorePlugin
        NpcPlugin,              // Depends on WorldPlugin
    ))
```

**Violation Consequences:** Panic on resource access or system ordering failure. Always verify plugin order before adding new plugins.

### Time Scaling Architecture (Critical System)

The codebase implements a **three-layer time abstraction** that is the foundation of all simulation:

#### Layer 1: Bevy's `Time` (Real Time)
- Raw frame deltas from the engine
- **Never use this directly in simulation code**

#### Layer 2: `SimulationClock` (Scaled Time)
- Location: [src/core/plugin.rs:75-80](src/core/plugin.rs#L75-L80)
- Multiplies real time by `time_scale` (default: 1.0, min: 0.001)
- Updated every frame by `update_simulation_clock` system
- **This is what ALL simulation systems should read**

```rust
pub fn tick(&mut self, real_delta: Duration) {
    self.last_real_delta = real_delta;
    self.last_scaled_delta = real_delta.mul_f32(self.time_scale);
    self.elapsed += self.last_scaled_delta;
}
```

#### Layer 3: `WorldClock` (Game Time)
- Location: [src/world/time.rs:165-172](src/world/time.rs#L165-L172)
- Day/night cycle derived from scaled time
- `time_of_day` is a fraction (0.0 = midnight, 0.5 = noon)
- Drives lighting, NPC schedules, and future weather systems

**Data Flow:**
```
Real Frame Delta (Bevy Time)
    ↓ [update_simulation_clock system]
SimulationClock.last_scaled_delta = real_delta * time_scale
    ↓ [advance_world_clock system]
WorldClock.time_of_day += scaled_delta / seconds_per_day
    ↓ [apply_world_lighting system]
Sun rotation & ambient light updated
    ↓ [update_schedule_state system]
NPC activities updated based on time of day
```

### System Ordering (Critical Path)

**WorldPlugin Systems** [src/world/plugin.rs:26-37](src/world/plugin.rs#L26-L37):
```rust
.add_systems(Update, (
    advance_world_clock,
    (
        update_cursor_grab,
        fly_camera_mouse_look.after(update_cursor_grab),
        fly_camera_translate,
    ),
    apply_world_lighting.after(advance_world_clock),
))
```

**NpcPlugin Systems** [src/npc/plugin.rs:17-18](src/npc/plugin.rs#L17-L18):
```rust
.add_systems(Startup, spawn_debug_npcs.after(spawn_world_environment))
.add_systems(Update, update_schedule_state.after(advance_world_clock))
```

**Critical Ordering Rules:**
1. `advance_world_clock` MUST run before `apply_world_lighting` (lighting reads `time_of_day`)
2. `update_cursor_grab` MUST run before `fly_camera_mouse_look` (cursor state affects input)
3. `spawn_debug_npcs` MUST run after `spawn_world_environment` (NPCs need ground plane)
4. `update_schedule_state` MUST run after `advance_world_clock` (schedules read `time_of_day`)

**Parallelization:** Systems without `.after()` / `.before()` constraints can run in parallel (Bevy handles this automatically).

### Configuration Pattern (Two-Phase Validation)

All configuration loading follows a defensive pattern to prevent crashes from invalid user input:

**Pattern:** [src/world/time.rs:74-95](src/world/time.rs#L74-L95)
1. **Load:** Read TOML file
2. **Parse:** Deserialize to private `RawTimeConfig` structs
3. **Validate:** Convert to public structs via `From` trait with clamping/normalization
4. **Fallback:** Use defaults on any error (with `warn!` logging)

**Example Validation Logic** [src/world/time.rs:103-110](src/world/time.rs#L103-L110):
```rust
let seconds_per_day = (clock.day_length_minutes.max(0.1)) * 60.0;  // Min 6 seconds/day
let sunrise = clock.sunrise_fraction.clamp(0.0, 1.0);               // Force [0, 1]
let sunset = clock.sunset_fraction.clamp(0.0, 1.0);
let (sunrise, sunset) = if sunrise == sunset {
    (sunrise, (sunrise + 0.5) % 1.0)  // Force separation if equal
} else {
    (sunrise.min(sunset), sunrise.max(sunset))  // Ensure sunrise < sunset
};
```

**Config File Location:** `config/time.toml` (relative to executable working directory, **no hot reload**)

**When Adding New Config:**
- Create `RawXxxConfig` (private) with `#[derive(Deserialize)]`
- Create `XxxSettings` (public) with validation in `impl From<RawXxxConfig>`
- Use `load_or_default()` pattern in plugin `build()`
- Always log loaded values for user verification

### Component Design Patterns

#### Pattern 1: Stateful Marker Components
[src/world/components.rs:5-22](src/world/components.rs#L5-L22)
```rust
#[derive(Component)]
pub struct FlyCamera {
    pub yaw: f32,
    pub pitch: f32,
    pub move_speed: f32,
    pub look_sensitivity: f32,
}
```
- Public fields for direct access (ECS philosophy)
- Constructor with sensible defaults
- Used as query filter + data storage

#### Pattern 2: Zero-Sized Markers
[src/world/components.rs:25-26](src/world/components.rs#L25-L26)
```rust
#[derive(Component, Default)]
pub struct PrimarySun;
```
- No data storage, pure query filter
- Derives `Default` for ergonomic spawning
- Used to identify specific entities (e.g., main directional light)

#### Pattern 3: Newtype Wrappers with Display
[src/npc/components.rs:7-19](src/npc/components.rs#L7-L19)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub struct NpcId(u64);

impl fmt::Display for NpcId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NPC-{:04}", self.0)  // Outputs "NPC-0001" format
    }
}
```
- Strong typing for semantic IDs (prevents mixing up different ID types)
- Custom Display for better logging
- Supports hashing for HashMap keys

### Query Patterns

#### Pattern 1: Single Mutable Query (Bevy 0.17)
[src/world/systems.rs:56](src/world/systems.rs#L56)
```rust
mut cursor_options: Single<&mut CursorOptions>
```
- `Single<T>` ensures exactly one entity matches
- Panics if 0 or >1 matches (intentional for unique resources)

#### Pattern 2: Optional Single Query
[src/world/systems.rs:87](src/world/systems.rs#L87)
```rust
if let Ok((fly_cam, mut transform)) = query.single_mut() {
    // Safe handling if entity doesn't exist
}
```

#### Pattern 3: Marker-Based Query
[src/world/time.rs:179](src/world/time.rs#L179)
```rust
mut sun_query: Query<(&PrimarySun, &mut Transform, &mut DirectionalLight)>
```
- `&PrimarySun` filters to tagged entity
- Avoids querying all lights in scene (performance)

### Mathematical Calculations (Key Insights)

#### Sun Rotation [src/world/time.rs:182-185](src/world/time.rs#L182-L185)
```rust
let sun_angle = (day_fraction - 0.25) * TAU;  // Subtract 0.25 so noon (0.5) is vertical
let declination = settings.sun_declination;    // Tilt for dawn/dusk bias
let rotation = Quat::from_euler(EulerRot::ZYX, 0.0, declination, sun_angle).normalize();
```
- `TAU` (2π) for full rotation
- `-0.25` offset makes 0.0 = 6am, 0.5 = noon (sun overhead)

#### Daylight Factor (Smooth Transitions) [src/world/time.rs:187-200](src/world/time.rs#L187-L200)
```rust
let daylight_span = settings.sunset_fraction - settings.sunrise_fraction;
let mut t = day_fraction;
if t < settings.sunrise_fraction {
    t += 1.0;  // Handle midnight wrap (crucial for continuous cycles)
}
let offset = (t - settings.sunrise_fraction) % 1.0;
let normalized = (offset / daylight_span).clamp(0.0, 1.0);
normalized.sin().max(0.0)  // Sine curve for smooth dawn/dusk
```
- Midnight wrapping prevents discontinuities
- Sine curve creates natural-looking transitions

#### Light Intensity (Power Curve) [src/world/time.rs:202-203](src/world/time.rs#L202-L203)
```rust
let intensity = settings.night_lux +
    (settings.noon_lux - settings.night_lux) * daylight_factor.powf(1.5);
```
- Power 1.5 exponent creates sharper twilight drop-off (more realistic)
- Lerps between 5.0 lux (moonlight) and 50,000.0 lux (sunlight)

## Coding Style and Conventions

### Naming Conventions

**Systems:**
- Verb phrases: `update_simulation_clock`, `spawn_debug_npcs`, `apply_world_lighting`
- No `_system` suffix (Bevy 0.17 convention)

**Components:**
- Noun phrases: `FlyCamera`, `Identity`, `PrimarySun`
- Adjective + Noun for markers: `PrimarySun` (not just `Sun`)

**Resources:**
- Noun phrases with context: `SimulationClock`, `WorldTimeSettings`
- Generator suffix for factories: `NpcIdGenerator`

**Constants:**
- SCREAMING_SNAKE_CASE: `DEFAULT_TIME_SCALE`, `MIN_TIME_SCALE`, `CONFIG_PATH`
- Include units in name: `day_length_minutes`, `sun_declination_radians`

### Resource vs Component Usage

**Resources (Global Singletons):**
- `SimulationClock` - Application-wide time scaling
- `WorldClock` - Shared day/night cycle
- `WorldTimeSettings` - Configuration data
- `NpcIdGenerator` - Monotonic ID issuer
- `AmbientLight` - Scene-wide lighting (Bevy built-in)

**Components (Per-Entity State):**
- `FlyCamera` - Camera-specific orientation
- `Identity` - NPC-specific data
- `DailySchedule` - Per-NPC schedule
- `ScheduleState` - Per-NPC activity tracking

**Rule of Thumb:** Use Resources for singleton state or shared configuration; Components for per-entity data.

### Error Handling Philosophy

**Configuration Errors:**
- Fail gracefully with `warn!` logging + default fallback
- **NEVER panic on invalid user input**
- Example: [src/world/time.rs:80-92](src/world/time.rs#L80-L92)

**Validation Strategy:**
- Clamp numeric inputs to safe ranges (`min`, `max`, `clamp`)
- Normalize invalid combinations (e.g., `sunrise == sunset`)
- Log assumptions made during validation
- Use `saturating_add` for overflow protection

**No `unwrap()` in Production Code:**
- Use `unwrap_or()`, `unwrap_or_default()`, or match expressions
- Exception: Tests can use `unwrap()` freely

## Critical Pitfalls to Avoid

### Pitfall 1: Plugin Order Violations
**Risk:** Adding plugin before its dependencies
**Symptom:** Panic on resource access (e.g., `SimulationClock` not found)
**Prevention:** Always check plugin order in [src/main.rs](src/main.rs) before adding new plugins

### Pitfall 2: Using Bevy's Time Instead of SimulationClock
**Risk:** Systems reading `Res<Time>` instead of `Res<SimulationClock>`
**Symptom:** Behavior doesn't respect time scale (runs at real-time speed)
**Prevention:** Lint rule or comment in module READMEs

**❌ BAD:**
```rust
fn bad_system(time: Res<Time>) {
    let delta = time.delta_secs();  // NOT SCALED!
}
```

**✅ GOOD:**
```rust
fn good_system(sim_clock: Res<SimulationClock>) {
    let delta = sim_clock.last_scaled_delta().as_secs_f32();
}
```

### Pitfall 3: Configuration Path Assumptions
**Risk:** Config paths relative to executable, not source
**Symptom:** Works in `cargo run`, fails in packaged build
**Prevention:** Document working directory requirements; consider embedded defaults via `include_str!`

### Pitfall 4: Unvalidated TOML
**Risk:** Adding new config fields without validation
**Symptom:** Invalid values crash simulation
**Prevention:** Always use two-phase pattern (Raw structs + `From` trait validation)

### Pitfall 5: Ignoring System Ordering
**Risk:** Forgetting `.after()` / `.before()` constraints
**Symptom:** Race conditions, flickering, incorrect state
**Prevention:** Document dependencies in comments; add integration tests

## Feature Flags

**Current Flags** [Cargo.toml:6-8](Cargo.toml#L6-L8):
```toml
[features]
default = []
core_debug = []
```

**Usage Pattern:**
- Conditional compilation: `#[cfg(feature = "core_debug")]`
- Dead code suppression: `#[cfg_attr(not(feature), allow(dead_code))]`
- Zero runtime cost when disabled (compiled out entirely)

**Debug Output** [src/core/plugin.rs:136-143](src/core/plugin.rs#L136-L143):
```
Sim elapsed: 42.15s | scale: 1.000 | real dt: 0.0167s | scaled dt: 0.0167s
```
- Logs every simulated second (not every frame)
- Shows time scale multiplier and both delta values

**When Adding New Debug Features:**
- Gate behind feature flag or config toggle
- Use `info!(target: "feature_name", "...")` for filterable logs
- Document in module README and CLAUDE.md

## Important Default Values

| Constant | Value | Location | Rationale |
|----------|-------|----------|-----------|
| `DEFAULT_TIME_SCALE` | 1.0 | [src/core/plugin.rs:7](src/core/plugin.rs#L7) | Real-time default for development |
| `MIN_TIME_SCALE` | 0.001 | [src/core/plugin.rs:8](src/core/plugin.rs#L8) | Prevents zero/negative time |
| `GROUND_SCALE` | 100.0 | [src/world/systems.rs:12](src/world/systems.rs#L12) | Large walkable area |
| `CAMERA_START_POS` | (-12, 8, 16) | [src/world/systems.rs:13](src/world/systems.rs#L13) | Angled overview of origin |
| `day_length_minutes` | 10.0 | [config/time.toml:4](config/time.toml#L4) | Fast cycles for testing |
| `sunrise_fraction` | 0.22 | [config/time.toml:6](config/time.toml#L6) | ~5:16 AM in 24h day |
| `sunset_fraction` | 0.78 | [config/time.toml:8](config/time.toml#L8) | ~6:43 PM in 24h day |
| `noon_lux` | 50,000.0 | [config/time.toml:14](config/time.toml#L14) | Realistic outdoor sunlight |
| `night_lux` | 5.0 | [config/time.toml:16](config/time.toml#L16) | Moonlit visibility |

## Development Guidelines

### Code Style Requirements
- Follow Rust 2021 edition conventions
- **PascalCase** for types/traits/enums, **snake_case** for modules/files
- Keep files under **400 lines**; split when new responsibilities emerge
- All commits must pass `cargo fmt` and `cargo clippy -D warnings`
- Public fields for Components (ECS philosophy), private fields for Resources

### Documentation Requirements
When making behavior changes, update:
1. `CHANGELOG.md` - Step-level history
2. `docs/tech_notes.md` - Technical decisions
3. Relevant `src/**/README.md` - Module documentation
4. `.agent/tasks.yaml` - Task tracking
5. `.agent/ai_memory.V.N.yaml` - Decisions and open questions

**Note:** The project follows an AI-driven development workflow coordinated through `.agent/` files. Review `AGENT.md` for the masterplan and active steps before making significant changes.

### Module READMEs
Each plugin module includes a README describing:
- Plugin responsibilities and components
- Integration notes and usage examples
- Follow-up tasks and design decisions

**Always read module READMEs before modifying unfamiliar code.**

### Test Coverage
**Current Status:** ~3% (2 tests in `src/core/plugin.rs`)

**Coverage by Module:**
- Core: 2 tests ✓ (time scaling math, clamping validation)
- World: 0 tests ✗
- NPC: 0 tests ✗

**Priority Test Gaps:**
1. `WorldClock::tick()` day wrapping at midnight
2. `apply_world_lighting()` calculation edge cases
3. NPC schedule transitions at midnight
4. Config validation edge cases (sunrise == sunset, etc.)

**When Adding Tests:**
- Unit tests in same file as implementation (`#[cfg(test)] mod tests`)
- Integration tests in `tests/` directory
- Use descriptive test names: `test_clock_scales_delta_with_multiplier`

## Extension Points for Future Development

### Plugin System
New plugins can hook into existing schedules (Startup, Update):
```rust
impl Plugin for DialoguePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LlmClient::new())
            .add_systems(Update, (
                process_llm_queue,
                update_dialogue_ui.after(process_llm_queue),
            ));
    }
}

// In main.rs:
.add_plugins((
    CorePlugin::default(),
    WorldPlugin,
    NpcPlugin,
    DialoguePlugin::default(),  // Can read Identity, WorldClock
))
```

### Planned Architecture (from AGENT.md)
Future modules that will be added:
- `/dialogue` - LLM client, prompt templates, chat UI (Milestone M4)
- `/economy` - Resources, jobs, production loops (Milestone M5)
- `/save` - SQLite persistence, migrations (Milestone M2)
- `/ui` - HUD, menus, panels (Milestone M4)
- `/weather` - Weather simulation and environmental effects (Milestone M6)
- `/mods` - Data pack loading (Milestone M9)

### Recommended Patterns for Future Systems

#### Pattern 1: Event-Driven Architecture
```rust
#[derive(Event)]
pub struct NpcActivityChanged {
    pub npc_id: NpcId,
    pub old_activity: String,
    pub new_activity: String,
}

// In update_schedule_state:
if state.current_activity != current_activity {
    events.send(NpcActivityChanged { /* ... */ });
}
```

#### Pattern 2: Change Detection (Performance)
```rust
fn update_lighting(
    clock: Res<WorldClock>,
    settings: Res<WorldTimeSettings>,
) {
    if !clock.is_changed() {
        return;  // Skip expensive calculations
    }
    // ...
}
```

#### Pattern 3: Tick-Rate Independence
```rust
#[derive(Resource)]
pub struct TickAccumulator {
    elapsed: Duration,
    tick_interval: Duration,
}

fn schedule_updates(
    mut accumulator: ResMut<TickAccumulator>,
    clock: Res<SimulationClock>,
) {
    accumulator.elapsed += clock.last_scaled_delta();
    while accumulator.elapsed >= accumulator.tick_interval {
        accumulator.elapsed -= accumulator.tick_interval;
        // Run schedule update
    }
}
```

## Debug Features

### Camera Controls
- **Mouse Look:** Hold right-click + drag
- **Movement:** WASD (horizontal), Space (up), LShift (down)
- **Sprint:** Hold LCtrl (2.5x speed multiplier: [src/world/systems.rs:115](src/world/systems.rs#L115))
- **Cursor Lock:** Automatic during right-click

### NPC Visualization
- Debug capsule meshes (0.3 radius, 1.0 height)
- Color-coded: Alric (red), Bryn (blue), Cedric (green)
- Positions: [src/npc/systems.rs:18-52](src/npc/systems.rs#L18-L52)

### Logging Levels
- `info!()` - Configuration, major state changes, NPC activities
- `warn!()` - Configuration fallbacks, validation issues
- `debug!()` - Not used (prefer feature-gated `info!` with targets)

## Build Profiles

**Development Profile** [Cargo.toml:16-21](Cargo.toml#L16-L21):
```toml
[profile.dev]
opt-level = 1  # Light optimization for our code

[profile.dev.package."*"]
opt-level = 3  # Full optimization for dependencies (Bevy)
```
- **Rationale:** Fast iteration on game code, performant Bevy rendering

**Release Profile** [Cargo.toml:24-27](Cargo.toml#L24-L27):
```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 1
```
- Aggressive optimization with link-time optimization (LTO)

## Important Notes

- **Bevy version:** 0.17 (pre-release track). Verify crate compatibility before adding dependencies.
- **Target platform:** Windows desktop first; Linux should work opportunistically.
- **Physics:** Deferred until needed. When required, prefer `bevy_rapier3d` with compatibility audit.
- **Current milestone:** M0 (Core scaffolding). Active step tracked in `AGENT.md` Section 4.
- **Working directory:** Config files expect to run from repository root (where `Cargo.toml` lives)

## Maintenance Checklist

**Before Each Commit:**
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy -- -D warnings` (MUST pass)
- [ ] Run `cargo test` (all tests pass)
- [ ] Test both `cargo run` and `cargo run --features core_debug`

**Before Adding Dependencies:**
- [ ] Verify Bevy 0.17 compatibility
- [ ] Check license compatibility
- [ ] Document rationale in `AGENT.md` Section 6

## Quick Reference

| Task | Command |
|------|---------|
| Run application | `cargo run` |
| Run with debug logs | `cargo run --features core_debug` |
| Format code | `cargo fmt` |
| Lint (strict) | `cargo clippy -- -D warnings` |
| Type check | `cargo check --all-targets` |
| Run tests | `cargo test` |
| Build docs | `cargo doc --no-deps` |
| Live reload | `cargo watch -x "check --all-targets"` |

## Key File Locations

- **Entry Point:** [src/main.rs](src/main.rs)
- **Time Scaling:** [src/core/plugin.rs](src/core/plugin.rs)
- **Day/Night Cycle:** [src/world/time.rs](src/world/time.rs)
- **Camera Controls:** [src/world/systems.rs](src/world/systems.rs)
- **NPC System:** [src/npc/systems.rs](src/npc/systems.rs)
- **Configuration:** [config/time.toml](config/time.toml)

## For Comprehensive Analysis

See [COMPREHENSIVE_CODEBASE_ANALYSIS.md](COMPREHENSIVE_CODEBASE_ANALYSIS.md) for:
- Complete architectural analysis with line-by-line explanations
- All mathematical calculations with rationale
- Code quality metrics and dependency graphs
- Performance considerations and optimization strategies
- Detailed extension points and scaling patterns

