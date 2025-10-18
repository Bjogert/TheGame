# Economy Module

The economy prototype now builds daily work plans from configuration rather than hard-coding a single trade loop. A small planner walks the recipe graph and converts each request into per-profession tasks.

- `EconomyRegistry` loads recipes and daily requests from `config/economy.toml`. Each recipe defines the actor profession, required inputs, and produced goods.
- `prepare_economy_day` creates requests (e.g., farmer needs tools) and the planner expands them into `ActorTask` entries per profession (`WaitForGood`, `Manufacture`, `Deliver`).
- `advance_actor_tasks` executes tasks once villagers reach their crates, waits naturally when inputs are missing, transfers inventory, and emits `TradeCompletedEvent`/dialogue prompts for deliveries.
- Deliveries only complete when both the courier and the recipient are stationed at their crates, ensuring trades stay grounded in visible locations.
- Placeholder goods (`TradeGoodPlaceholder`) spawn beside crates while inventory stacks exist. Visuals come from `TradeGoodPlaceholderVisuals`, so goods linger until consumed or traded away.
- `EconomyDependencyMatrix` still maps wellbeing categories to goods. After tasks complete, daily snapshots emit `ProfessionDependencyUpdateEvent` so motivation systems can react to shortages or satisfied needs.

The configuration-driven approach keeps behaviour extensible while we iterate on more professions and goods. Design notes for broader expansion live in docs/economy_blueprint.md.

## Module Layout
- `systems/spawning.rs` creates crate entities and registers placeholder visuals.
- `systems/day_prep.rs` rebuilds daily task queues once per world day, clearing the previous plan when requests change.
- `systems/task_execution.rs` advances queued tasks, manipulates inventories, and emits dependency updates.
- `systems/dialogue.rs` converts trade progress into dialogue requests so the broker sees planner output.
- Shared constants (placeholder offsets, profession labels) live at the top of the relevant modules to avoid ad-hoc literals.
