# NPC Conversation Behavior Plan - S1.17

**Status:** PLANNING
**Created:** 2025-10-26
**Goal:** Make NPCs stop moving and face each other during dialogue, like real people

---

## Current Problem

**Timing Mismatch:**
1. Trade completes instantly â†’ `send_trade_and_dialogue()` fires
2. Dialogue request enqueued immediately
3. OpenAI API call takes 2-3 seconds
4. DialogueResponseEvent emitted
5. Dialogue panel appears
6. **BUT: NPCs keep walking during steps 2-5**

**Result:** By the time dialogue appears, NPCs have walked away from each other. Feels unnatural and disconnected.

## Timing Analysis

**Current Distances & Travel Times:**
- Farmer (8, 0.2, 3) â†’ Miller (0, 0.2, -6.5): ~12.4 units = **5.0 seconds**
- Farmer â†’ Blacksmith (-6, 0.2, 1.5): ~14.1 units = **5.6 seconds**
- Miller â†’ Blacksmith: ~10 units = **4.0 seconds**
- NPC move speed: **2.5 units/second**
- OpenAI API call: **2-3 seconds**

**Key Insight:** NPCs take 4-6 seconds to walk between crates, but API call only takes 2-3 seconds. We can start the API call **while they're walking** so dialogue is ready when they arrive!

---

## Desired Behavior

**Optimized Conversation Flow (Predictive API Call):**
1. **Approach:** NPC A starts walking toward NPC B's crate
2. **Predict:** Calculate travel time: distance / 2.5 units/sec
3. **Queue Early:** If travel time > 2 seconds, enqueue dialogue request immediately
4. **Travel:** NPC walks for 4-6 seconds (API call completes during travel)
5. **Arrive:** NPC reaches crate, dialogue response already cached
6. **Stop:** Both NPCs stop moving (add InConversation component)
7. **Face:** NPCs turn to face each other smoothly
8. **Speak:** Dialogue panel appears **instantly** (no awkward waiting)
9. **Hold:** NPCs remain stopped/facing for 10s dialogue lifetime
10. **Resume:** After dialogue despawns, NPCs continue their tasks

**Key Optimization:** Start API call when movement begins, not when it completes. Dialogue is ready by arrival time.

**Visual Reference:** User mentioned YouTube video showing NPCs stopping and facing each other naturally during conversation.

---

## Architecture Design

### 1. New Component: `InConversation`

```rust
/// Tracks when an NPC is engaged in a dialogue conversation
#[derive(Component, Debug, Clone)]
pub struct InConversation {
    pub partner: NpcId,           // Who they're talking to
    pub request_id: DialogueRequestId,  // Track which request this is for
    pub started_at: f32,          // World time when conversation started
    pub predicted_arrival: f32,   // World time when both NPCs should be face-to-face
    pub state: ConversationState,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationState {
    Approaching,         // Walking toward partner, API call in progress
    WaitingAtDestination, // Arrived, waiting for API response
    Speaking,            // Dialogue panel is visible
}
```

**Location:** `src/npc/components.rs`

**State Transitions:**
- `Approaching`: NPC starts walking, dialogue request enqueued, `predicted_arrival` set from travel-time estimate
- `WaitingAtDestination`: NPC arrives at crate before API responds
- `Speaking`: DialogueResponseEvent received, panel visible

### 2. Modified Locomotion System

**File:** `src/npc/systems.rs::drive_npc_locomotion`

**Change:** Allow movement during `Approaching`, freeze during `WaitingAtDestination` and `Speaking`

```rust
pub fn drive_npc_locomotion(
    sim_clock: Res<SimulationClock>,
    mut movers: Query<(
        &Identity,
        &mut Transform,
        &mut NpcLocomotion,
        Option<&mut InConversation>
    )>,
    world_transforms: Query<&GlobalTransform>,
) {
    for (identity, mut transform, mut locomotion, conversation) in movers.iter_mut() {
        // Freeze if in conversation (but not approaching)
        if let Some(conv) = conversation {
            if conv.state != ConversationState::Approaching {
                continue;  // Skip movement for waiting/speaking NPCs
            }
        }

        // Existing locomotion logic...
        // When arrival detected:
        if distance <= arrive_distance {
            // ... existing arrival logic ...

            // NEW: Transition Approaching â†’ WaitingAtDestination
            if let Some(mut conv) = conversation {
                if conv.state == ConversationState::Approaching {
                    commands.entity(entity).insert(InConversation {
                        state: ConversationState::WaitingAtDestination,
                        ..conv.clone()
                    });
                }
            }
        }
    }
}
```

### 3. New System: `orient_conversing_npcs`

**Purpose:** Make NPCs face their conversation partner (only when stopped)

```rust
pub fn orient_conversing_npcs(
    time: Res<Time>,
    mut npcs: Query<(&Identity, &mut Transform, &InConversation)>,
    all_npcs: Query<(&Identity, &Transform)>,
) {
    for (identity, mut transform, conversation) in npcs.iter_mut() {
        // Only orient when stopped (not while approaching)
        if conversation.state == ConversationState::Approaching {
            continue;
        }

        // Find partner's position
        let Some(partner_transform) = all_npcs
            .iter()
            .find(|(id, _)| id.id == conversation.partner)
            .map(|(_, t)| t)
        else {
            warn!("InConversation partner not found: {}", conversation.partner);
            continue;
        };

        // Calculate look direction (Y-axis rotation only, no vertical tilt)
        let direction = Vec3::new(
            partner_transform.translation.x - transform.translation.x,
            0.0,  // No vertical look
            partner_transform.translation.z - transform.translation.z,
        );

        if direction.length() < 0.01 {
            continue;  // Too close, skip rotation
        }

        let direction = direction.normalize();

        // Calculate target rotation (rotate around Y axis to face direction)
        let angle = direction.x.atan2(direction.z);
        let target_rotation = Quat::from_rotation_y(angle);

        // Smoothly rotate toward partner (slerp for natural turn)
        transform.rotation = transform.rotation.slerp(target_rotation, 5.0 * time.delta_secs());
    }
}
```

**Location:** `src/npc/systems.rs`

### 4. Conversation Lifecycle Coordination

#### Start Conversation (Trade Dialogue)

**File:** `src/economy/systems/dialogue.rs::send_trade_and_dialogue`

**Predictive enqueue + state priming:**
```rust
pub(super) fn send_trade_and_dialogue(
    trade_writer: &mut MessageWriter<TradeCompletedEvent>,
    queue: &mut DialogueRequestQueue,
    npc_lookup: &NpcLookup,      // NEW: map NpcId -> (Entity, Transform, move_speed)
    commands: &mut Commands,     // NEW
    world_clock: &WorldClock,    // NEW
    input: TradeDialogueInput,
) {
    // ... existing trade event ...

    if let (Some(speaker), Some(target)) = (input.from, input.to) {
        // Resolve entities + motion data
        let (speaker_entity, speaker_tf, speaker_speed) =
            npc_lookup.expect(speaker, "speaker entity missing");
        let (target_entity, target_tf, _target_speed) =
            npc_lookup.expect(target, "target entity missing");

        // Estimate travel duration in the XZ plane
        let distance = speaker_tf.translation().xz().distance(target_tf.translation().xz());
        let travel_seconds = distance / speaker_speed.max(0.1);
        let now = world_clock.day_fraction();
        let predicted_arrival = now + travel_seconds;

        // Build dialogue request immediately so the API call runs while they walk
        // ... existing request construction ...
        let request_id = queue.enqueue(request);

        // Decide initial state. Long walks stay in Approaching, short hops wait in place.
        let initial_state = if travel_seconds >= MIN_PREDICTIVE_TRAVEL_SECS {
            ConversationState::Approaching
        } else {
            ConversationState::WaitingAtDestination
        };

        // Stamp both NPCs so locomotion/orientation systems react
        for (entity, partner) in [
            (speaker_entity, target),
            (target_entity, speaker),
        ] {
            commands.entity(entity).insert(InConversation {
                partner,
                request_id,
                started_at: now,
                predicted_arrival,
                state: initial_state,
            });
        }
    }
}
```

**Notes:**
- `NpcLookup::expect` is a helper that returns `(Entity, GlobalTransform, move_speed)` for an `NpcId` (log a warning if missing).
- `MIN_PREDICTIVE_TRAVEL_SECS` ˜ 2.0 so we only kick off predictive dialogue when the walk is longer than the API latency; short hops drop straight into `WaitingAtDestination`.
- When both NPCs enter `ConversationState::Approaching`, locomotion keeps them moving; once they arrive the locomotion system promotes the state to `WaitingAtDestination`.


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
4. Test: Full conversation cycle from trade â†’ stop â†’ face â†’ speak â†’ resume

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
   - **Mitigation:** Consider caching NpcId â†’ Entity mapping in a resource

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
1. Run game, trigger trade â†’ verify NPCs stop moving
2. Verify NPCs face each other (observe rotation)
3. Verify dialogue panel appears while NPCs are stopped
4. Verify NPCs resume movement after panel despawns
5. Test rapid trades (do NPCs queue conversations properly?)

### Automated Tests (Future):
- Unit test: InConversation component lifecycle
- Integration test: Conversation state transitions
- System test: Full trade â†’ dialogue â†’ resume flow

---

## Documentation Updates Required

- `CHANGELOG.md` - Add S1.17 entry for conversation behavior
- `TASK.md` - Add S1.17 step with exit criteria
- `README.md` - Update active features list
- `.agent/ai_memory.V.1.yaml` - Document decision rationale
- `.agent/tasks.yaml` - Track implementation tasks

---

## Exit Criteria

âœ… NPCs stop moving when trade dialogue is triggered
âœ… NPCs face their conversation partner (Y-axis rotation)
âœ… NPCs remain stopped while waiting for OpenAI response
âœ… NPCs remain stopped while dialogue panel is visible
âœ… NPCs resume normal movement after dialogue panel despawns
âœ… No ghost conversations (components cleaned up properly)
âœ… Zero clippy warnings, code formatted
âœ… Tested with live trade interactions

---

## Future Enhancements (Out of Scope for S1.17)

- **Animation System:** Play "talking" animation during conversations
- **Gesture System:** NPCs gesture/emote while speaking
- **Conversation Groups:** Support 3+ NPC conversations
- **Conversation Interruption:** Player can interrupt NPC conversations
- **Lip Sync:** Match dialogue text to mouth movements (ambitious!)

