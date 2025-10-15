# Economy Module

The current economy slice is a placeholder "micro trade loop" that proves inventories, schedules, and dialogue integrations work together.

- `EconomyPlugin` assigns professions to the debug NPCs after they spawn and runs a daily trade loop once the world clock advances.
- `TradeCompletedEvent` records production, processing, and exchange steps. Dialogue systems listen for these events to build conversation context.
- `Inventory` and simple `TradeGood` enums keep track of crate-style goods passed between the farmer, miller, and blacksmith.
- Each exchange queues a dialogue request with trade context, ensuring NPC chatter references the latest activity.

This loop is intentionally small; expect it to be replaced by a data-driven economy once Step 7 begins.

Design for the transition lives in `docs/economy_blueprint.md`, which introduces an `EconomyRegistry`, work-order queues, and an expanded event taxonomy to bridge economy, NPC schedules, and dialogue.
