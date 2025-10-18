# Module Responsibility Map

_Last updated: 2025-10-16_

This snapshot summarises how the major gameplay plugins relate to one another. It focuses on
what each module owns, which cross-plugin resources/events it touches, and any ordering
constraints to remember while wiring new systems.

## Core
- **Responsibilities:** Owns `CorePlugin`, registers `SimulationClock`, and provides optional
  debug logging for scaled ticks.
- **Provides:**
  - `SimulationClock` resource (scaled time deltas consumed by downstream systems).【F:src/core/plugin.rs†L26-L115】
- **Consumes:** Bevy's `Time` resource to tick the simulation clock.【F:src/core/plugin.rs†L117-L124】
- **Cross-plugin notes:**
  - Must be added before `WorldPlugin`, `NpcPlugin`, `EconomyPlugin`, and `DialoguePlugin` so its
    `SimulationClock` resource exists when they start.
  - Any system that should respect time scaling should read `Res<SimulationClock>` instead of
    raw `Time`.

## World
- **Responsibilities:** Loads time-of-day settings, maintains the `WorldClock`, spawns the
  ground plane/lighting/camera rig, and updates lighting each tick.【F:src/world/plugin.rs†L1-L39】
- **Provides:**
  - `WorldTimeSettings` resource loaded from `config/time.toml`.【F:src/world/time.rs†L11-L93】
  - `WorldClock` resource exposing day count and fractional time-of-day.【F:src/world/time.rs†L109-L169】
  - `PrimarySun` marker and environment entities through startup systems.【F:src/world/plugin.rs†L5-L30】
- **Consumes:**
  - `SimulationClock` (to advance the world clock each frame).【F:src/world/time.rs†L168-L176】
- **Cross-plugin notes:**
  - `WorldClock` feeds daily scheduling in the economy systems and NPC motivation evaluation.
  - Startup ordering ensures world environment spawns before NPC and economy setup systems that
    rely on crate/camera placement.

## NPC
- **Responsibilities:** Spawns debug villagers, runs schedule ticking, drives locomotion, and
  maintains motivation state/rewards.【F:src/npc/plugin.rs†L1-L42】
- **Provides:**
  - Resources: `NpcIdGenerator`, `ScheduleTicker`, `DailyDependencyTracker`, `MotivationConfig`.
  - Components: `NpcLocomotion`, `Identity`, `Profession` assignments (added during economy setup).【F:src/npc/components.rs†L24-L145】【F:src/economy/systems.rs†L62-L114】
  - Events: `NpcActivityChangedEvent` for schedule transitions.【F:src/npc/events.rs†L8-L14】
- **Consumes:**
  - `WorldClock` to timestamp schedule changes and dependency evaluations.【F:src/npc/systems.rs†L78-L129】【F:src/npc/motivation/systems.rs†L96-L152】
  - `SimulationClock` for motivation decay cadence.【F:src/npc/motivation/systems.rs†L211-L238】
  - Economy events/resources: `TradeCompletedEvent`, `ProfessionDependencyUpdateEvent`,
    `EconomyDependencyMatrix`, profession crates/inventory data.【F:src/npc/motivation/systems.rs†L5-L210】
  - Dialogue events: `DialogueResponseEvent` for social rewards.【F:src/npc/motivation/systems.rs†L63-L96】
- **Cross-plugin notes:**
  - Motivation systems react to economy and dialogue outputs, so event/message types must remain
    stable when iterating on those modules.
  - `spawn_debug_npcs` runs after the world environment and before economy profession assignment
    so crates/NPCs line up correctly.【F:src/npc/plugin.rs†L30-L37】

## Dialogue
- **Responsibilities:** Hosts the `DialogueRequestQueue`, rate limiting, telemetry logging, and
  broker abstraction for LLM providers.【F:src/dialogue/plugin.rs†L1-L53】
- **Provides:**
  - Resources: `DialogueRateLimitConfig`, `DialogueRateLimitState`, `DialogueRequestQueue`,
    `ActiveDialogueBroker`, `DialogueTelemetry`, `DialogueTelemetryLog`.【F:src/dialogue/plugin.rs†L20-L44】
  - Events: `DialogueResponseEvent`, `DialogueRequestFailedEvent`.【F:src/dialogue/plugin.rs†L33-L52】
- **Consumes:**
  - Uses Bevy `Time` to advance queue cooldowns.【F:src/dialogue/queue.rs†L108-L128】
  - Processes requests pushed by economy systems and (future) gameplay code via `DialogueRequestQueue`.【F:src/economy/systems.rs†L250-L773】
- **Cross-plugin notes:**
  - Economy task systems enqueue trade/schedule prompts; NPC motivation listens for dialogue
    responses to award social gains.
  - The active broker defaults to the OpenAI implementation but can be swapped by injecting a
    different boxed `DialogueBroker` implementation during startup.【F:src/dialogue/plugin.rs†L36-L39】

## Economy
- **Responsibilities:** Loads economy configs, spawns profession crates, assigns placeholder
  professions, plans daily tasks, and executes production/trade loops.【F:src/economy/plugin.rs†L1-L61】【F:src/economy/systems.rs†L1-L249】
- **Provides:**
  - Resources: `EconomyRegistry`, `EconomyDependencyMatrix`, `ActorTaskQueues`, `EconomyDayState`,
    profession crate/placeholder registries.【F:src/economy/plugin.rs†L20-L47】
  - Events: `TradeCompletedEvent`, `ProfessionDependencyUpdateEvent`.【F:src/economy/plugin.rs†L29-L47】【F:src/economy/events.rs†L13-L63】
- **Consumes:**
  - `WorldClock` for day progression and dependency evaluation cadence.【F:src/economy/systems.rs†L120-L212】【F:src/economy/systems.rs†L260-L360】
  - NPC data: `Identity`, `Profession`, `NpcLocomotion` to drive task execution and crate travel.【F:src/economy/systems.rs†L13-L251】【F:src/economy/systems.rs†L360-L597】
  - Dialogue resources: `DialogueRequestQueue` to enqueue flavour prompts for trades/schedules.【F:src/economy/systems.rs†L250-L773】
- **Cross-plugin notes:**
  - Startup ordering ensures profession crates spawn after the world environment and before NPC
    locomotion kicks in.【F:src/economy/plugin.rs†L32-L47】
  - `TradeCompletedEvent` drives NPC motivation rewards; `ProfessionDependencyUpdateEvent`
    captures wellbeing coverage for each profession and feeds the motivation tracker.【F:src/npc/motivation/systems.rs†L90-L189】

---

Keep this map synced as new plugins appear or cross-module contracts change so future contributors
can trace dependencies without spelunking through every file.
