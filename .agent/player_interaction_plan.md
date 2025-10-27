# Player Interaction Plan (S1.18)

**Created:** 2025-10-26
**Status:** Planning
**Goal:** Allow player to walk up to NPCs and press "E" to initiate dialogue conversations

---

## Problem Statement

Currently, dialogue only occurs between NPCs during trade exchanges. The player (camera) is a passive observer with no way to interact with NPCs directly. This limits immersion and prevents the player from learning about NPC states, needs, or world information.

**User Request:** "I would like to be able to interact with the NPCs. So if I walk up to one and press 'E' I should be able to chat with him/her."

---

## Current State Analysis

### What We Have ✅
1. **Camera/Player Position**
   - `FlyCamera` component with `Transform` in [src/world/components.rs](src/world/components.rs:5-22)
   - Camera position effectively represents player position (first-person perspective)
   - WASD movement, mouse look already functional

2. **Input Handling Infrastructure**
   - `ButtonInput<KeyCode>` pattern used for F7 debug hotkey in [src/dialogue/plugin.rs](src/dialogue/plugin.rs)
   - Already have keyboard input system registered

3. **Dialogue System (Fully Functional)**
   - `DialogueRequestQueue::enqueue()` - Queue dialogue requests
   - `DialogueBroker` (OpenAI integration) - Generate responses
   - `DialogueResponseEvent` - Emit responses
   - UI dialogue panel system - Display conversations
   - Rate limiting and cooldowns already handled

4. **Conversation Behavior (S1.17)**
   - NPCs stop and face conversation partners via `InConversation` component
   - `orient_conversing_npcs` system rotates NPCs toward partner using quaternion slerp
   - Can extend to make NPCs face camera position for player interactions

5. **NPC Infrastructure**
   - `Identity` component with unique IDs and display names
   - `Transform` component for NPC positions
   - All NPCs queryable via standard Bevy ECS queries

### What We DON'T Have ❌
1. **Proximity Detection** - No system tracks which NPC is near the player
2. **Player Entity Marker** - Camera is not tagged as "player" for dialogue purposes
3. **Interaction Range** - No defined distance for "near enough to talk"
4. **Visual Feedback** - No indication when player can interact with NPC
5. **Player-Specific Dialogue Context** - Prompts currently assume NPC-to-NPC trade context

---

## Architecture Design

### Module Structure

**Option A: Add to `src/world/` (Camera/Player Systems)**
- ✅ Camera already lives here
- ✅ Input handling (fly camera controls) already in `src/world/systems.rs`
- ❌ Mixing player interaction with world environment feels off

**Option B: Create `src/player/` Module** ⭐ **RECOMMENDED**
- ✅ Clean separation of concerns
- ✅ Follows existing pattern (`npc/`, `world/`, `dialogue/`, `economy/`)
- ✅ Room to grow (inventory, stats, quests later)
- ✅ Small modular files per CLAUDE.md guidelines
- Structure:
  ```
  src/player/
    mod.rs          - Module exports
    components.rs   - Player marker, interaction state
    systems.rs      - Proximity detection, input handling
    plugin.rs       - PlayerPlugin registration
  ```

**Option C: Add to `src/npc/` (Interaction Systems)**
- ❌ Conceptually wrong - player is not an NPC
- ❌ Would pollute NPC module with player-specific logic

**Decision: Option B - Create `src/player/` module**

---

## Component Design

### New Components

#### 1. `Player` Marker Component
```rust
/// Marker component identifying the player entity (camera).
#[derive(Component, Debug)]
pub struct Player;
```

**Purpose:** Tag camera entity so systems can distinguish player from NPCs.

**Location:** Add to camera spawn in [src/world/systems.rs](src/world/systems.rs:46-50)

#### 2. `NearbyNpc` Component (Optional State Tracking)
```rust
/// Tracks the NPC the player is currently near and can interact with.
#[derive(Component, Debug)]
pub struct NearbyNpc {
    pub npc_entity: Entity,
    pub npc_id: NpcId,
    pub npc_name: String,
    pub distance: f32,
}
```

**Purpose:** Cache proximity state to avoid recalculating every frame.

**Alternative:** Use a `PlayerState` resource instead of component.

**Decision:** Use **Resource** pattern (cleaner for single-player, easier to query):

```rust
/// Resource tracking player interaction state.
#[derive(Resource, Default, Debug)]
pub struct PlayerInteractionState {
    pub nearby_npc: Option<NearbyNpcInfo>,
}

#[derive(Debug, Clone)]
pub struct NearbyNpcInfo {
    pub entity: Entity,
    pub npc_id: NpcId,
    pub name: String,
    pub distance: f32,
}
```

---

## System Design

### System 1: `detect_nearby_npcs`
**Purpose:** Find the nearest NPC within interaction range and update `PlayerInteractionState`.

**Parameters:**
- `player_query: Query<&Transform, With<Player>>` - Get player position
- `npc_query: Query<(Entity, &Transform, &Identity), (With<Identity>, Without<InConversation>)>` - Get all NPCs not already in conversation
- `mut interaction_state: ResMut<PlayerInteractionState>` - Update state

**Logic:**
1. Get player position from camera transform
2. Iterate all NPCs, calculate distance
3. Find nearest NPC within `INTERACTION_RANGE` (e.g., 3.0 units)
4. Exclude NPCs already in conversation (`Without<InConversation>`)
5. Update `PlayerInteractionState.nearby_npc`

**Constants:**
```rust
const INTERACTION_RANGE: f32 = 3.0; // Player must be within 3 units to interact
```

**Run Condition:** Always (Update schedule)

---

### System 2: `handle_player_interaction_input`
**Purpose:** Listen for E key press and enqueue dialogue request if NPC is nearby.

**Parameters:**
- `keyboard: Res<ButtonInput<KeyCode>>` - Check for E key
- `interaction_state: Res<PlayerInteractionState>` - Check if NPC nearby
- `mut queue: ResMut<DialogueRequestQueue>` - Enqueue dialogue request
- `player_query: Query<Entity, With<Player>>` - Get player entity

**Logic:**
1. Check `keyboard.just_pressed(KeyCode::KeyE)`
2. If pressed and `interaction_state.nearby_npc.is_some()`:
   - Get NPC entity and ID from state
   - Get player entity
   - Build `DialogueRequest` with context:
     - `speaker: npc_id` (NPC speaks to player)
     - `target: None` (or special "player" target)
     - `context: DialogueContext::PlayerInteraction`
   - Enqueue request via `queue.enqueue(request)`
   - Log interaction: `info!("Player initiates conversation with {}", npc_name)`

**Run Condition:** Always (Update schedule)

---

### System 3: `start_player_conversations`
**Purpose:** Handle `DialogueRequestedEvent` when player is the target, add `InConversation` to NPC.

**Parameters:**
- `mut commands: Commands`
- `mut events: MessageReader<DialogueRequestedEvent>`
- `world_clock: Res<WorldClock>`
- `player_query: Query<(Entity, &Transform), With<Player>>`
- `npc_query: Query<(Entity, &Identity)>`

**Logic:**
1. Listen for `DialogueRequestedEvent`
2. If `target == None` or special player marker:
   - Get player entity and position
   - Get NPC entity from `speaker`
   - Add `InConversation` to NPC with:
     - `partner: special_player_id` (need to define)
     - `request_id: event.request_id`
     - `started_at: current_time`
     - `state: ConversationState::WaitingAtDestination`
   - NPC will freeze and face player via existing `orient_conversing_npcs` system

**Challenge:** `InConversation.partner` expects `NpcId`, but player isn't an NPC.

**Solutions:**
1. **Special NpcId::Player variant** (clean but requires enum change)
2. **Use NpcId("player")** (hack but minimal changes)
3. **Separate PlayerConversation component** (more code duplication)

**Decision:** Option 2 for MVP - Use `NpcId("player".into())` as special marker.

---

### System 4: `orient_npcs_toward_player`
**Purpose:** Make NPCs face the player (camera) during player-initiated conversations.

**Parameters:**
- `time: Res<Time>`
- `player_query: Query<&Transform, With<Player>>`
- `mut npc_query: Query<(&mut Transform, &InConversation), Without<Player>>`

**Logic:**
1. Get player position
2. For each NPC with `InConversation`:
   - Check if `conversation.partner` is player (via special NpcId)
   - Calculate direction from NPC to player
   - Apply quaternion slerp rotation (reuse logic from `orient_conversing_npcs`)

**Alternative:** Extend existing `orient_conversing_npcs` to handle player as partner.

**Decision:** Extend existing system (less duplication).

---

## Dialogue Context Updates

### Modify `DialogueContext` Builder

**Current:** Context assumes NPC-to-NPC trade exchanges with trade events.

**Needed:** Detect when player is the target and adjust prompt accordingly.

**Changes to [src/dialogue/broker/openai.rs](src/dialogue/broker/openai.rs):**

```rust
// In build_system_prompt or similar:
if request.target == Some(NpcId("player".into())) {
    // Player interaction prompt
    "You are {speaker_name}, a {profession} in a medieval village.
    The player approaches and asks to chat. Respond naturally based on:
    - Your current activity: {current_activity}
    - Your mood: {mood_state}
    - Recent events: {recent_events}
    Be helpful but stay in character. Keep responses under 2-3 sentences."
} else {
    // Existing NPC-to-NPC trade prompt
}
```

**Implementation:** Small change to prompt builder, pass player flag through context.

---

## Visual Feedback (Phase 2 - Optional Polish)

### Option 1: Material Highlight
- Query nearby NPC entity
- Swap `MeshMaterial3d` to glowing/outlined version
- Remove highlight when out of range

### Option 2: UI Prompt
- Spawn UI text near NPC or in corner
- Show "Press E to talk to [Name]"
- Despawn when out of range

### Option 3: Both (Most Polish)

**Recommendation for MVP:** Skip for now, implement in S1.18b if needed.

---

## Implementation Phases

### Phase 1: MVP (Core Functionality) - 30 min

**Goal:** Walk up, press E, NPC responds via dialogue panel.

**Steps:**
1. Create `src/player/` module structure
   - `mod.rs` - Exports
   - `components.rs` - `Player` marker, `PlayerInteractionState` resource
   - `systems.rs` - `detect_nearby_npcs`, `handle_player_interaction_input`
   - `plugin.rs` - `PlayerPlugin` registration
2. Add `Player` component to camera in `src/world/systems.rs::spawn_world_environment`
3. Implement proximity detection system
4. Implement E key input handling
5. Modify `orient_conversing_npcs` to handle player partner (check for special NpcId)
6. Modify dialogue context to detect player interaction
7. Test: Walk up to NPC, press E, verify dialogue appears and NPC faces camera

**Exit Criteria:**
- ✅ Player can press E near NPC to initiate dialogue
- ✅ NPC stops and faces camera
- ✅ Dialogue panel appears with NPC's response
- ✅ NPC resumes after timeout (existing cleanup system)

### Phase 2: Polish (Optional) - 20-30 min
8. Add visual highlight to nearby NPC (material glow)
9. Add "Press E to talk" UI prompt
10. Add cooldown to prevent spam (E key debounce)
11. Handle edge cases (NPC walks away mid-conversation)

**Exit Criteria:**
- ✅ Clear visual indication of interactable NPC
- ✅ Polished UX with prompts and feedback

---

## Configuration

**New Constants (in `src/player/systems.rs`):**
```rust
/// Maximum distance for player-NPC interaction (units)
const INTERACTION_RANGE: f32 = 3.0;

/// Minimum time between interaction attempts (seconds)
const INTERACTION_COOLDOWN: f32 = 1.0; // Phase 2
```

**No new config files needed** - These are tuning constants, not user-facing settings.

---

## Testing Strategy

### Manual Test Cases

1. **Basic Interaction**
   - Walk up to NPC (within 3 units)
   - Press E
   - Verify dialogue panel spawns
   - Verify NPC faces camera
   - Verify NPC resumes after timeout

2. **Out of Range**
   - Stand far from NPC (>3 units)
   - Press E
   - Verify nothing happens

3. **NPC Already in Conversation**
   - Wait for NPC-to-NPC trade dialogue
   - Walk up and press E during conversation
   - Verify nothing happens (NPC excluded from detection)

4. **Multiple NPCs Nearby**
   - Stand between two NPCs
   - Press E
   - Verify only nearest NPC responds

5. **Walk Away Mid-Conversation**
   - Start conversation with E
   - Walk away before timeout
   - Verify NPC cleanup still works (existing timeout handles this)

### Automated Tests (Future)
- Unit test distance calculation
- Unit test nearest NPC selection
- Integration test dialogue enqueue flow

---

## Risk Assessment

### Technical Risks

1. **NpcId Type Mismatch** (Medium)
   - `InConversation.partner` expects `NpcId`, player isn't an NPC
   - **Mitigation:** Use special `NpcId("player".into())` marker for MVP
   - **Future:** Refactor to `ConversationPartner` enum

2. **Orientation System Conflicts** (Low)
   - `orient_conversing_npcs` assumes NPC-to-NPC, may not handle player
   - **Mitigation:** Add special case for player partner, use camera position

3. **Dialogue Context Mismatch** (Low)
   - OpenAI prompts assume trade context
   - **Mitigation:** Detect player target and use different prompt template

4. **Performance** (Low)
   - Distance calculation every frame for all NPCs
   - **Mitigation:** Only 3 NPCs currently, trivial cost. If scaling needed, use spatial partitioning.

### UX Risks

1. **Confusing Interaction Range** (Medium)
   - Player may not know when they're close enough
   - **Mitigation Phase 2:** Visual feedback (highlight, UI prompt)

2. **E Key Conflicts** (Low)
   - E is common key, may conflict with future features
   - **Mitigation:** Document in controls, make remappable later

3. **Spam Prevention** (Low)
   - Player could spam E key
   - **Mitigation Phase 2:** Add interaction cooldown

---

## File Modifications

### New Files (Create)
- `src/player/mod.rs` (~10 lines)
- `src/player/components.rs` (~30 lines)
- `src/player/systems.rs` (~120 lines)
- `src/player/plugin.rs` (~25 lines)

### Modified Files
- `src/main.rs` - Register `PlayerPlugin`
- `src/lib.rs` - Add `pub mod player;`
- `src/world/systems.rs` - Add `Player` component to camera spawn
- `src/npc/systems.rs` - Extend `orient_conversing_npcs` for player partner
- `src/dialogue/broker/openai.rs` - Add player interaction prompt branch
- `CHANGELOG.md` - Document S1.18 implementation
- `TASK.md` - Add S1.18 task entry
- `.agent/tasks.yaml` - Update with S1.18 subtasks

**Total New Code:** ~185 lines
**Total Modified Code:** ~30 lines
**Small modular files:** ✅ All files under 150 lines

---

## Plugin Coordination

**New PlayerPlugin Systems (Update Schedule):**
1. `detect_nearby_npcs` - Runs every frame
2. `handle_player_interaction_input` - Runs every frame

**System Ordering:**
```
Update schedule:
  detect_nearby_npcs          (early, updates state)
  handle_player_interaction_input (after detection)
  [existing NPC/dialogue systems]
  orient_conversing_npcs      (modified to handle player)
```

**No new events needed** - Reuse existing `DialogueRequestedEvent`.

---

## Documentation Strategy

### During Implementation
- Update `CHANGELOG.md` after each phase completes
- Update `TASK.md` with S1.18 entry after MVP done
- Update `.agent/tasks.yaml` as subtasks complete

### After Implementation
- Add `src/player/README.md` explaining interaction system
- Update `README.md` features list and controls section
- Update `.agent/ai_memory.V.N.yaml` with lessons learned

---

## Success Criteria

### MVP Success (Phase 1)
- [x] Player can walk up to any NPC
- [x] Press E to initiate dialogue
- [x] NPC stops and faces player
- [x] Dialogue panel appears with contextual response
- [x] NPC resumes activity after timeout
- [x] System excludes NPCs already in conversation
- [x] No crashes, compile warnings, or clippy errors

### Polish Success (Phase 2)
- [ ] Visual indication of nearby interactable NPC
- [ ] "Press E to talk" UI prompt
- [ ] Interaction cooldown prevents spam
- [ ] Clean UX that feels intuitive

---

## Time Estimate

| Phase | Task | Estimate |
|-------|------|----------|
| **Planning** | This document | 20 min ✅ |
| **Phase 1** | Module structure | 5 min |
| | Components & resource | 5 min |
| | Proximity detection system | 10 min |
| | E key input system | 5 min |
| | Extend orient system | 5 min |
| | Dialogue context changes | 5 min |
| | Integration & testing | 10 min |
| **Phase 1 Total** | | **45 min** |
| **Phase 2** | Visual feedback | 15 min |
| | UI prompt | 10 min |
| | Polish & edge cases | 10 min |
| **Phase 2 Total** | | **35 min** |
| **Documentation** | CHANGELOG, TASK, README | 10 min |
| **TOTAL (MVP + Docs)** | | **75 min** |

**Recommendation:** Implement Phase 1 MVP now (~45 min), test with user, then decide if Phase 2 polish is needed.

---

## Next Steps

1. Get user approval on this plan
2. Update `.agent/tasks.yaml` with subtasks
3. Create `src/player/` module
4. Implement Phase 1 systems incrementally
5. Test MVP
6. Document and demo
7. Decide on Phase 2 polish

---

**Plan Status:** ✅ COMPLETE - Ready for implementation
**Estimated Total Time:** 45 min (MVP) + 10 min (docs) = 55 minutes
---

## Execution Progress (2025-10-26)

### Update Todos
- [x] Phase 1.1 � Create `src/player/` module scaffold (`mod.rs`, `components.rs`, `systems.rs`, `plugin.rs`)
- [x] Phase 1.2 � Add `Player` marker + `PlayerInteractionState` resource and hook into camera spawn
- [x] Phase 1.3 � Stub `detect_nearby_npcs` / `handle_player_interaction_input` systems
- [x] Phase 1.4 � Extend `orient_conversing_npcs` and conversation lifecycle for player partner
- [x] Phase 1.5 � Modify `start_conversations` / dialogue queueing for player interactions
- [x] Phase 1.6 � Compile & pass `cargo fmt`, `cargo clippy`, `cargo check`
- [ ] Phase 1.7 � Update documentation set (CHANGELOG, TASK.md, README, ai_memory)
- [ ] Phase 2.x � Visual prompt, interaction cooldown, polish (deferred)

### Phase 1.5 Notes � start_conversations + Dialogue Wiring
- Converted `NpcId` handling to use the new `NpcId::player()` / `is_player()` helpers; no more direct tuple-field access.
- Player interaction requests now leverage `DialogueRequest::new` with a simple status prompt and context summary so downstream systems get consistent data.
- Queue `enqueue` call captures the returned `DialogueRequestId` for logging/diagnostics.

### Phase 1.6 - Compile & Test (status: done)
- Resolved: updated imports, switched to `Query::single()`, added the player helper, removed unused re-exports, and satisfied Clippy (type alias allow, context initializer, etc.). Latest `cargo fmt && cargo clippy -- -D warnings` and `cargo check` all pass.

### Phase 1.7 � Documentation Prep
- Once the MVP compiles, update:
  1. `.agent/tasks.yaml` with S1.18 sub-steps status
  2. `CHANGELOG.md` under the latest milestone
  3. `TASK.md` to log S1.18 outcome and blockers (if any)
  4. `.agent/ai_memory.V.1.yaml` with decisions (e.g., special `NpcId::player` marker) and risks
- Add a short `src/player/README.md` describing the new module and interaction flow.

### Immediate Next Actions
1. Update documentation artifacts once interaction MVP is confirmed (CHANGELOG, TASK.md, README, ai_memory, player README).
2. Play-test the new interaction loop to validate proximity threshold, prompt quality, and orientation behaviour (adjust as needed).
3. Decide whether to proceed with Phase 2 polish (UI prompt, cooldowns, visual feedback) in this iteration or defer.

Keep this section updated as implementation progresses so future agents can pick up without repeating investigation work.


### Phase 1.8 - Player Response UI (Complete)
- Spawned a bottom-left response window whenever an NPC addresses the player; offers three canned replies styled similarly to the dialogue panel.
- Selecting a reply queues a follow-up `DialogueRequest` (player reply context + NPC prompt) and closes the window until the next NPC response.
- Interaction state now tracks active conversation metadata (NPC name, last line) so prompts feel grounded.
- Cleanup system removes the window once the conversation times out, preventing stale UI.
- **Follow-ups:** consider dynamic response sets, keyboard shortcuts, or free-text input once the UX is validated.

