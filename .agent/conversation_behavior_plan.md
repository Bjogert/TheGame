# NPC Conversation Behavior Plan - S1.17

**Status:** PLANNING
**Created:** 2025-10-26
**Goal:** Make NPCs stop moving and face each other during dialogue, like real people

---

## Current Problem

**Timing Mismatch:**
1. Trade completes instantly → `send_trade_and_dialogue()` fires
2. Dialogue request enqueued immediately
3. OpenAI API call takes 1-3 seconds
4. DialogueResponseEvent emitted
5. Dialogue panel appears
6. **BUT: NPCs keep walking during steps 2-5**

**Result:** By the time dialogue appears, NPCs have walked away from each other. Feels unnatural and disconnected.

---

## Desired Behavior

**Natural Conversation Flow:**
1. **Pre-conversation:** NPC finishes trade, about to speak
2. **Stop:** Both NPCs stop moving
3. **Face:** NPCs turn to face each other
4. **Wait:** Hold position while waiting for API response
5. **Speak:** Dialogue panel appears (10s lifetime)
6. **Resume:** After dialogue despawns, NPCs continue their tasks

**Visual Reference:** User mentioned YouTube video showing NPCs stopping and facing each other naturally during conversation.

---

## Architecture Design

### 1. New Component: `InConversation`

```rust
/// Tracks when an NPC is engaged in a dialogue conversation
#[derive(Component, Debug, Clone)]
pub struct InConversation {
    pub partner: NpcId,           // Who they're talking to
    pub started_at: f32,          // World time when conversation started
    pub state: ConversationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationState {
    WaitingForResponse,  // Queued dialogue request, waiting for API
    Speaking,            // Dialogue panel is visible
}
```

**Location:** `src/npc/components.rs`

### 2. Modified Locomotion System

**File:** `src/npc/systems.rs::drive_npc_locomotion`

**Change:** Skip movement for NPCs with `InConversation` component

```rust
pub fn drive_npc_locomotion(
    sim_clock: Res<SimulationClock>,
    mut movers: Query<
        (&Identity, &mut Transform, &mut NpcLocomotion),
        Without<InConversation>  // ← NEW: Skip NPCs in conversation
    >,
    world_transforms: Query<&GlobalTransform>,
) {
    // Existing locomotion logic unchanged
}
```

### 3. New System: `orient_conversing_npcs`

**Purpose:** Make NPCs face their conversation partner

```rust
pub fn orient_conversing_npcs(
    time: Res<Time>,
    mut npcs: Query<(&Identity, &mut Transform, &InConversation)>,
    all_transforms: Query<&Transform, Without<InConversation>>,
) {
    for (identity, mut transform, conversation) in npcs.iter_mut() {
        // Find partner's position
        let partner_pos = find_npc_position(conversation.partner, &all_transforms);

        // Calculate look direction (Y-axis rotation only)
        let direction = Vec3::new(
            partner_pos.x - transform.translation.x,
            0.0,  // No vertical look
            partner_pos.z - transform.translation.z,
        ).normalize();

        // Smoothly rotate toward partner (slerp for natural turn)
        let target_rotation = Quat::from_rotation_y(direction.angle_between(Vec3::NEG_Z));
        transform.rotation = transform.rotation.slerp(target_rotation, 5.0 * time.delta_secs());
    }
}
```

**Location:** `src/npc/systems.rs`

### 4. Conversation Lifecycle Coordination

#### Start Conversation (Trade Dialogue)

**File:** `src/economy/systems/dialogue.rs::send_trade_and_dialogue`

**After enqueuing dialogue request:**
```rust
pub(super) fn send_trade_and_dialogue(
    trade_writer: &mut MessageWriter<TradeCompletedEvent>,
    queue: &mut DialogueRequestQueue,
    commands: &mut Commands,  // ← NEW parameter
    world_clock: &WorldClock,  // ← NEW parameter
    input: TradeDialogueInput,
) {
    // ... existing trade event ...

    if let (Some(speaker), Some(target)) = (input.from, input.to) {
        // ... existing dialogue queueing ...
        let id = queue.enqueue(request);

        // NEW: Mark both NPCs as in conversation
        let current_time = world_clock.day_fraction();
        commands.entity(speaker_entity).insert(InConversation {
            partner: target,
            started_at: current_time,
            state: ConversationState::WaitingForResponse,
        });
        commands.entity(target_entity).insert(InConversation {
            partner: speaker,
            started_at: current_time,
            state: ConversationState::WaitingForResponse,
        });
    }
}
```

**Challenge:** Need to pass entity handles, not just NpcIds. This requires refactoring `send_trade_and_dialogue` call sites.

#### Update Conversation State (Response Arrives)

**File:** `src/ui/dialogue_panel/systems.rs::spawn_dialogue_panel`

**After spawning panel:**
```rust
pub fn spawn_dialogue_panel(
    mut commands: Commands,
    mut tracker: ResMut<DialoguePanelTracker>,
    settings: Res<DialoguePanelSettings>,
    mut events: MessageReader<DialogueResponseEvent>,
    npc_query: Query<(Entity, &Identity, Option<&InConversation>)>,
) {
    for event in events.read() {
        // ... existing panel spawn logic ...

        // NEW: Update conversation state to Speaking
        let speaker_entity = find_npc_entity(event.response.speaker, &npc_query);
        if let Ok((entity, _, Some(mut conversation))) = npc_query.get_mut(speaker_entity) {
            commands.entity(entity).insert(InConversation {
                state: ConversationState::Speaking,
                ..conversation.clone()
            });
        }

        // Same for target if present
        if let Some(target_id) = event.response.target {
            let target_entity = find_npc_entity(target_id, &npc_query);
            if let Ok((entity, _, Some(mut conversation))) = npc_query.get_mut(target_entity) {
                commands.entity(entity).insert(InConversation {
                    state: ConversationState::Speaking,
                    ..conversation.clone()
                });
            }
        }
    }
}
```

#### End Conversation (Panel Despawns)

**File:** `src/ui/dialogue_panel/systems.rs::update_dialogue_panel`

**When despawning panel:**
```rust
pub fn update_dialogue_panel(
    mut commands: Commands,
    time: Res<Time>,
    mut tracker: ResMut<DialoguePanelTracker>,
    mut panel_query: Query<(Entity, &mut DialoguePanel)>,
    mut npc_query: Query<(Entity, &Identity, &InConversation)>,
    mut background_query: Query<&mut BackgroundColor>,
) {
    for (entity, mut panel) in panel_query.iter_mut() {
        panel.tick(time.delta());

        if panel.is_finished() {
            // NEW: Clear InConversation from both NPCs
            let speaker_entity = find_npc_entity(panel.npc_id(), &npc_query);
            commands.entity(speaker_entity).remove::<InConversation>();

            if let Some(target_id) = panel.target_id() {  // Need to add target tracking to DialoguePanel
                let target_entity = find_npc_entity(target_id, &npc_query);
                commands.entity(target_entity).remove::<InConversation>();
            }

            // Existing panel despawn
            tracker.active_panel = None;
            commands.entity(entity).despawn();
            continue;
        }

        // ... existing fade logic ...
    }
}
```

**Challenge:** `DialoguePanel` doesn't currently store the target NPC ID. Need to add this.

---

## Implementation Plan

### Phase 1: Component & Locomotion (15 min)
1. Add `InConversation` component to `src/npc/components.rs`
2. Add `ConversationState` enum
3. Modify `drive_npc_locomotion` query to exclude `InConversation` NPCs
4. Test: NPCs should freeze when component added manually

### Phase 2: Orientation Behavior (20 min)
1. Implement `orient_conversing_npcs` system in `src/npc/systems.rs`
2. Add system registration in `src/npc/plugin.rs`
3. Helper: `find_npc_by_id()` to locate NPC entities
4. Test: NPCs should face each other when InConversation added

### Phase 3: Conversation Lifecycle - Start (25 min)
1. Refactor `send_trade_and_dialogue` signature to accept Commands + WorldClock
2. Update all call sites to provide entity handles (may need economy systems refactor)
3. Insert `InConversation` components when dialogue queued
4. Test: NPCs stop and face each other immediately after trade

### Phase 4: Conversation Lifecycle - Update & End (25 min)
1. Add `target_id: Option<NpcId>` to `DialoguePanel` component
2. Update `spawn_dialogue_panel` to transition conversation state to Speaking
3. Update `update_dialogue_panel` to remove InConversation when despawning
4. Test: Full conversation cycle from trade → stop → face → speak → resume

### Phase 5: Edge Cases & Polish (15 min)
1. Handle conversation timeout (if API never responds, clear after 10s?)
2. Handle conversation interruption (new dialogue arrives before old one finishes)
3. Handle missing conversation partner (partner despawned mid-conversation)
4. Add logging for conversation state transitions

**Total Estimate:** 100 minutes (~1.5 hours)

---

## Alternative Approaches Considered

### Option A: Simple "Pause Movement" Flag
**Pros:** Simpler, just a bool flag
**Cons:** Doesn't track conversation partner, can't implement facing behavior
**Decision:** Rejected - we need partner tracking for orientation

### Option B: Dialogue-Driven (Wait for Response Before Moving)
**Pros:** No new components, minimal changes
**Cons:** NPCs would be frozen UNTIL dialogue appears (1-3s delay looks weird)
**Decision:** Rejected - need to support "waiting" state

### Option C: Animation-Based (Play "talking" animation)
**Pros:** Most realistic, proper animation states
**Cons:** No animation system yet, would delay this feature significantly
**Decision:** Deferred - implement basic stop/face first, add animations later

---

## Risks & Unknowns

1. **Entity Lookup Performance:** Finding NPC entities by ID in hot systems (spawn/update panels)
   - **Mitigation:** Consider caching NpcId → Entity mapping in a resource

2. **Conversation Stacking:** What if NPC receives 2 dialogue requests simultaneously?
   - **Mitigation:** DialogueRequestQueue already handles rate limiting, shouldn't happen

3. **Conversation Interruption:** What if NPC's schedule changes mid-conversation?
   - **Mitigation:** Schedule system should respect InConversation (skip transitions)

4. **API Timeout:** If OpenAI never responds, NPC stuck in WaitingForResponse forever
   - **Mitigation:** Add timeout (10s?) to clear InConversation if no response

5. **Pathfinding Conflicts:** Currently no pathfinding, but future system may conflict
   - **Mitigation:** Document that InConversation overrides pathfinding destinations

---

## Testing Strategy

### Manual Tests:
1. Run game, trigger trade → verify NPCs stop moving
2. Verify NPCs face each other (observe rotation)
3. Verify dialogue panel appears while NPCs are stopped
4. Verify NPCs resume movement after panel despawns
5. Test rapid trades (do NPCs queue conversations properly?)

### Automated Tests (Future):
- Unit test: InConversation component lifecycle
- Integration test: Conversation state transitions
- System test: Full trade → dialogue → resume flow

---

## Documentation Updates Required

- `CHANGELOG.md` - Add S1.17 entry for conversation behavior
- `TASK.md` - Add S1.17 step with exit criteria
- `README.md` - Update active features list
- `.agent/ai_memory.V.1.yaml` - Document decision rationale
- `.agent/tasks.yaml` - Track implementation tasks

---

## Exit Criteria

✅ NPCs stop moving when trade dialogue is triggered
✅ NPCs face their conversation partner (Y-axis rotation)
✅ NPCs remain stopped while waiting for OpenAI response
✅ NPCs remain stopped while dialogue panel is visible
✅ NPCs resume normal movement after dialogue panel despawns
✅ No ghost conversations (components cleaned up properly)
✅ Zero clippy warnings, code formatted
✅ Tested with live trade interactions

---

## Future Enhancements (Out of Scope for S1.17)

- **Animation System:** Play "talking" animation during conversations
- **Gesture System:** NPCs gesture/emote while speaking
- **Conversation Groups:** Support 3+ NPC conversations
- **Conversation Interruption:** Player can interrupt NPC conversations
- **Lip Sync:** Match dialogue text to mouth movements (ambitious!)
