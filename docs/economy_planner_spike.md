
# Economy Planner Refactor Plan

## Goal
Replace the hard-coded farmer → miller → blacksmith loop with a needs-driven work/order system that scales to more professions, recipes, and multi-hop trades while keeping behaviour readable and natural.

---

## 1. Data Model

- **Recipes (config):**
  - Move existing production steps into a TOML config, e.g.:
    ```toml
    [[recipes]]
    id = "grain_harvest"
    actor = "farmer"
    produces = [{ good = "grain", quantity = 1 }]
    consumes = []
    location = "crate"

    [[recipes]]
    id = "flour_milling"
    actor = "miller"
    produces = [{ good = "flour", quantity = 1 }]
    consumes = [{ good = "grain", quantity = 1 }]
    location = "crate"
    ```
  - Each recipe defines: actor profession, inputs, outputs, location hint to keep movement simple, and optional duration (default instant today).
- **Needs (config):**
  - Extend the dependency matrix so each profession lists category requirements and accepted goods per category (already partially there).
- **Economy registry (resource):**
  - Load recipes & profession needs at startup into an `EconomyRegistry` resource.
  - Provide helper lookups: recipes by actor, by product, goods satisfying categories.

---

## 2. Work Orders

- Introduce a `WorkOrder` struct:
  - `id`, `requester` (NpcId or Profession), `category` (need being satisfied), `target_good`, `quantity`, `status` (`Pending`, `InProgress`, `Blocked`, `Completed`), `assigned_to`.
  - Optional `depends_on: Vec<WorkOrderId>` for chaining sub-orders.
- Resource `WorkOrderQueue`:
  - Holds active orders, issues new IDs, and exposes queries (e.g., open orders by profession or category).
  - Emits events when orders progress to keep motivation/telemetry in sync.
- **Order lifecycle:**
  1. At day start, each profession checks needs (via dependency matrix) and creates orders for unsatisfied categories (e.g., farmer: `Tools` → `target_good = tools`).
  2. Order gets assigned to the profession that can produce the good (`EconomyRegistry` lookup). The assignee becomes responsible for fulfilling it.
  3. If the assignee lacks inputs, they spawn sub-orders for each missing input (e.g., blacksmith needs flour).
  4. Orders complete when goods are delivered to the requester’s inventory.

---

## 3. Planner

- Add a simple planner that resolves a `WorkOrder` into actions:
  - Input: order + current inventories.
  - Search over recipes (breadth-first) to find a sequence producing the target good from available inventory or further sub-orders.
  - Output: `PlanStep` list, e.g. `Produce grain -> Deliver grain to miller -> Wait for flour -> Produce tools -> Deliver tools to farmer`.
  - Each step indicates required location (crate) and resulting good transfers.
- Plans should be recomputed if conditions change (e.g., inputs arrive). To keep it simple, recompute at the start of each tick for the leading order if the existing plan stalled.

---

## 4. Execution Systems

- **Daily orchestration system:**
  - Runs after NPCs reach their crates.
  - Triggers order creation for unmet needs.
  - Kicks off plan building for new orders.
- **Action execution system:**
  - Reads the current step of each active plan.
  - Moves the NPC to required location (crate or target NPC) using existing locomotion.
  - Once at location, performs the step:
    - Production steps: execute recipe (consume inputs, produce outputs, play placeholder spawn).
    - Delivery steps: ensure target NPC is present; if not, mark order `Blocked` and retry later. When both are present, transfer inventory and update placeholders.
  - Marks step complete and advances plan. When the final step finishes, mark order `Completed`, update dependency satisfaction, and clear placeholders if goods are consumed.
- **Waiting logic:**
  - If the next step needs another NPC who is away from their crate, leave order in `Blocked` and log the reason. NPC can idle or tackle other orders later (future refinement).

---

## 5. Systems & Resources Changes

- Replace existing `process_micro_trade_loop` with:
  1. `prepare_daily_orders` (run once per day).
  2. `assign_pending_orders` (match orders with producers).
  3. `advance_work_orders` (executes plan steps; handles movement and trade logic).
- Keep `TradeCompletedEvent` for telemetry but emit it when delivery steps finish instead of inline during hard-coded sequence.
- Keep placeholders in sync via the new execution flow (spawn when inventory transitions from 0 → >0, despawn on >0 → 0).
- Add debug logging for order creation, assignment, blocking, and completion so designers can trace behaviour.

---

## 6. Documentation & Telemetry

- Update `src/economy/README.md` with the new order/planner flow and configuration files.
- Extend `docs/tech_notes.md` to describe the needs-driven system and how to add recipes/orders.
- Refresh planning docs (TASK.md, AGENT.md summaries) once the refactor lands.
- Consider adding a `WorkOrderDebugEvent` for quick UI overlays later.

---

## 7. Incremental Delivery Plan

1. **Scaffold data structures**: `EconomyRegistry`, `WorkOrder`, `WorkOrderQueue`, load config.
2. **Write planner prototype**: given target good, derive action chain using existing three professions.
3. **Replace trade loop execution**: drive actions through the new plan, keeping current behaviour identical but no hard-coded sequence.
4. **Add blocking/wait handling**: ensure trades pause naturally if partners are absent.
5. **Expand docs/tests**: unit tests for planner search, order lifecycle, placeholder sync.
6. **Cleanup**: remove legacy code paths, update docs/telemetry accordingly.

This structured path keeps the simulation readable, unlocks future professions/goods, and supports more human-like behaviour (waiting, multi-step bartering) without another rewrite later.

