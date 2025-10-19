
This document explains, in everyday language, what we are trying to build, what "done" looks like for this phase of the project, and the main steps that carry us from today to that destination.


## The destination
We are steering the game toward a living medieval settlement that runs even when the player is not watching. The short-term goal is to have a small slice of that world running smoothly: villagers wake up, follow simple routines, talk to each other, and react believably to the time of day. Hitting that target gives us confidence that the underlying simulation, dialogue hooks, and timekeeping all work together before we attempt anything more ambitious.

## How we will get there
1. **Polish the simulation core.** Make sure the game loop, time scaling, and debugging tools are rock solid so that later systems have a dependable foundation.
2. **Grow the world slice.** Flesh out the environment with lighting, terrain, and a controllable camera so we can observe the settlement at different hours.
3. **Strengthen persistence.** Introduce saving and loading so the world can pause and resume without losing track of what happened.
4. **Deepen NPC behavior.** Expand villager schedules, needs, and identities so their daily routines feel intentional rather than random.
5. **Open the door for dialogue.** Wire up the dialogue broker so characters can exchange lines that reference their current situation.
6. **Prove a micro trade loop.** Before a full economy milestone, stand up a farmer → miller → blacksmith exchange so dialogue can react to real trades.
7. **Lay economy groundwork.** Define basic resources and jobs so villagers have reasons to move, gather, craft, and trade. Capture a resource dependency matrix that shows which professions rely on (and supply) each input so balancing stays readable as new characters arrive.
8. **Model villager motivation.** Land a dopamine-style wellbeing bar that rewards productive or social moments, decays over time, reacts to coping tools like alcohol, and feeds mood changes back into behaviour, dialogue, and production quality.

## What finishing this plan looks like
We will consider this phase complete once a small cast of villagers can go through a day: the sun rises and sets, routines adjust accordingly, conversations reflect what characters are doing, and the simulation survives a save-and-load cycle. At that point we will have the confidence to scale outward—adding weather, more complex economies, and eventually multiplayer—because the basic loop is already proving itself in a believable slice of life.

### Current status
- Step 5 (dialogue broker prototype) is implemented with a queued broker and rate limiting.
- Step 6 (micro trade loop checkpoint) remains stable: the farmer, miller, and blacksmith trade crates each day and broadcast events to dialogue.
- Step 7 (economy foundations) now has a published blueprint in `docs/economy_blueprint.md`, detailing config-driven professions, work orders, expanded events, and the upcoming resource dependency matrix.
- Step 8 (motivation spike) landed: dopamine meters, mood thresholds, alcohol boosts/hangovers, and dependency-driven rewards now drive villager wellbeing after each in-game day rolls over.
- Step 9 (dialogue telemetry persistence) landed: dialogue responses and failures now stream to `logs/dialogue_history.jsonl` for offline review alongside the in-memory telemetry buffer.
- Step 10 (economy & dialogue literal audit) finished: OpenAI defaults, trade placeholder offsets, and locomotion tolerances live in shared constants/config instead of scattered literals.
- Step 11 (systems modularisation) finished: the economy systems now live under `systems::{spawning, day_prep, task_execution, dialogue}` and the dialogue broker under `broker/{mod,config,openai}`.
- Step 12 (dead code sweep) finished: redundant helpers/imports removed after enabling `clippy -D dead_code`, keeping the reorganised modules clean for upcoming UI telemetry work.
- Upcoming focus: verify OpenAI integration (S1.13), add contextual dialogue triggers (S1.14), layer needs-driven decision making (S1.15), and surface conversations with world-space speech bubbles (S1.16).
