# Economy Foundation Blueprint (Step S1.5)

This document captures the outcomes of Step S1.5: outlining how the placeholder
micro trade loop will evolve into the first sustainable economy slice for
Milestone M5. It summarises the current behaviour, establishes guiding goals for
Step 7, and records design decisions, risks, and open questions uncovered while
reviewing the existing systems.

## 1. Current Snapshot
- The `EconomyPlugin` runs once per in-game day after `WorldClock` advances.
- Professions (`Farmer`, `Miller`, `Blacksmith`) are injected into the three
  debug NPCs during startup.
- Inventories are in-memory vectors that carry crate-style goods between
  professions, with no persistence.
- `TradeCompletedEvent` emits production, processing, and exchange moments;
  dialogue systems translate each trade into contextual conversation prompts.
- Rate limiting, retry logic, and scheduling currently live in `DialoguePlugin`.

## 2. Goals for Step 7 (Economy Foundations)
1. **Data-driven professions and goods**
   - Define professions, recipes, and trade goods through config files so they
     can be tuned without recompiling.
   - Support seasonal modifiers pulled from `WorldClock` to vary outputs.
2. **Persistent inventories & ledgers**
   - Track stock levels and trade history in a resource suitable for
     serialisation (preparing for Milestone M2 persistence work).
   - Surface aggregate data (production vs. consumption) for debugging and
     future UI overlays.
3. **Task scheduling & workloads**
   - Introduce work orders that NPC schedulers can consume, bridging the economy
     into the behaviour systems.
   - Allow multiple participants per job (e.g., hauling, crafting) with clear
     ownership of input and output goods.
4. **Dialogue integration hooks**
   - Preserve trade context events but expand them to cover order assignments,
     shortages, and fulfilment delays.
5. **Observability & balancing**
   - Emit structured telemetry (events or tracing spans) for production and
     shortages.
   - Add developer cheats (feature flagged) to inspect or tweak economy values
     at runtime.

## 3. Architectural Decisions
- **Economy Registry Resource**: Introduce an `EconomyRegistry` resource that
  holds collections of `ProfessionDefinition`, `Recipe`, and `TradeGood` data.
  This registry loads from TOML in `/config/economy/`. Each definition includes
  identifiers, labels, inputs/outputs, and optional seasonal modifiers.
- **Work Order Queue**: Create a `WorkOrder` struct referencing a recipe,
  assigned profession(s), location hints, and deadlines. Store these in a
  `WorkOrderQueue` resource processed each day tick.
- **Inventory Component Upgrade**: Replace the ad-hoc `Vec` storage with a
  `HashMap<TradeGoodId, Quantity>` backed by a lightweight serialisable type.
  Provide helper methods for deltas and borrow-check friendly queries.
- **Event Taxonomy**: Expand `TradeCompletedEvent` into a small set of economy
  events (`ProductionEvent`, `TransferEvent`, `ShortageEvent`) so dialogue and
  UI systems can react with finer granularity.
- **Economy Schedule Systems**: Split the monolithic day loop into systems:
  1. `generate_work_orders` (runs after `WorldClock` tick)
  2. `assign_work_orders` (coordinates with NPC schedule subsystem)
  3. `execute_work_orders` (mutates inventories and emits events)
  4. `settle_accounts` (records ledger entries and clears state)
- **Configuration Binding**: Provide default configs and guard invalid data with
  warnings plus fallback recipes to keep the simulation running.

## 4. Dependencies & Interfaces
- **NPC Module**: Needs a way to request and claim work orders; propose an
  `NpcWorkload` component updated by the schedule systems.
- **Dialogue Module**: Continue consuming economy events, but extend context
  structures to include shortage reasons, outstanding debts, and fulfilment
  status.
- **Save Module** (future): Should serialise inventories, pending orders, and
  ledger entries.
- **World Module**: Supplies seasonal/time-of-day modifiers used during recipe
  evaluation.

## 5. Risks & Mitigations
- **Config Explosion**: Large TOML files may become unwieldy. Mitigate by
  splitting per profession and adding schema validation tests.
- **NPC Scheduling Complexity**: Integrating work orders with existing schedule
  ticks could cause ordering issues. Prototype with a dual-resource approach
  (economy proposes, NPC confirms) before merging loops.
- **Data Races**: Multiple systems mutating inventories may conflict.
  Centralise inventory mutations in `execute_work_orders` and expose read-only
  snapshots elsewhere.
- **Performance**: HashMap inventories add overhead. Keep crate counts low and
  profile once more NPCs join.

## 6. Open Questions
- How should we represent spatial logistics (delivery routes) before we have a
  pathfinding solution?
- Do we need a price or barter system immediately, or can we defer valuation
  until after basic resource flow works?
- Should shortages trigger automatic work order generation, or remain manual for
  now?
- Which data belongs in the save-game vs. regenerated on load (e.g., work order
  templates)?

## 7. Next Actions
- Draft TOML schemas and example configs for professions and recipes.
- Prototype the `EconomyRegistry` and `WorkOrderQueue` resources behind feature
  flags before replacing the micro loop.
- Coordinate with the NPC team to align on how schedules will claim and report
  work order progress.
- Add integration tests capturing a full order lifecycle once the new systems
  land.
