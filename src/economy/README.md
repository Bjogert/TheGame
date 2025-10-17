# Economy Module

The current economy slice is a placeholder "micro trade loop" that proves inventories, schedules, and dialogue integrations work together.

- `EconomyPlugin` assigns professions to the debug NPCs after they spawn and runs a daily trade loop once the world clock advances.
- `TradeCompletedEvent` records production, processing, and exchange steps. Dialogue systems listen for these events to build conversation context.
- `Inventory` and simple `TradeGood` enums keep track of crate-style goods passed between the farmer, miller, and blacksmith.
- Each exchange queues a dialogue request with trade context, ensuring NPC chatter references the latest activity.
- Profession-specific crates now spawn at world start. The micro loop orders each profession to travel to its crate before processing work, so trades trigger only after NPCs visibly arrive.
- Goods now manifest as small placeholder boxes beside each profession crate. They appear when inventories gain matching items and disappear again once the stack is depleted, giving a quick visual read on local stock without changing the loop's authoritative inventory data.
- `dependency.rs` defines a placeholder dependency matrix that maps professions and goods to wellbeing categories. Daily trade snapshots emit `ProfessionDependencyUpdateEvent` so NPC motivation systems can reward satisfied needs or penalise shortages, only crediting categories when matching goods are present in inventory.

This loop is intentionally small; expect it to be replaced by a data-driven economy once Step 7 begins.

Design for the transition lives in `docs/economy_blueprint.md`, which introduces an `EconomyRegistry`, work-order queues, and an expanded event taxonomy to bridge economy, NPC schedules, and dialogue.
