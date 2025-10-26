# UI Panel Dialogue System - Implementation Plan

**Date:** 2025-10-26
**Task:** S1.16c - Replace world-space speech bubbles with UI panel dialogue system
**Estimated Time:** 30-60 minutes
**Priority:** High (blocks S1.14 conversational triggers)

---

## Context & Rationale

### Why We're Pivoting

**Previous Approach (S1.16a):** World-space Text2d entities above NPCs
- ❌ Camera2d doesn't properly project 3D world coordinates in Bevy 0.17
- ❌ bevy_mod_billboard (proper solution) only supports Bevy 0.14
- ❌ Spent ~2 hours debugging without success
- ❌ Would require weeks to build custom billboard rendering system

**New Approach (S1.16c):** UI panel with dialogue text
- ✅ Standard Bevy UI (NodeBundle) - well-tested, reliable
- ✅ Better UX: always visible, readable, organized
- ✅ Simpler implementation: ~30-60 min vs days/weeks
- ✅ Extensible: easy to add history, portraits, choices later
- ✅ Industry precedent: Disco Elysium, Divinity OS2, BG3, The Sims

---

## Design Decisions

### 1. Panel Position: Bottom-Right Corner

**Rationale:**
- Doesn't obscure NPCs (camera typically looks down/center)
- Common pattern in dialogue-heavy games
- Easy to ignore when not needed (peripheral vision)
- Leaves bottom-left free for future inventory/status UI

**Layout:**
```
┌─────────────────────────────────────────────┐
│                                             │
│         3D World View (NPCs, environment)   │
│                                             │
│                                             │
│                  ┌────────────────────────┐ │
│                  │ 💬 Farmer Alric        │ │
│                  │ "Good harvest this     │ │
│                  │  year! The grain is    │ │
│                  │  ready for trade."     │ │
│                  └────────────────────────┘ │
└─────────────────────────────────────────────┘
```

### 2. Panel Behavior: Single Active Dialogue

**Rationale:**
- Simpler implementation (no scrolling, no history management)
- Matches simulation-first philosophy (live events, not logs)
- Can add history later if needed (S1.16d)
- Reduces screen clutter

**Timing:**
- Auto-advance after `lifetime_seconds` (reuse SpeechBubbleSettings value)
- Fade-out during final `fade_seconds`
- New dialogue replaces old immediately (like speech bubbles did)

### 3. Panel Content: NPC Name + Dialogue Text

**MVP Components:**
- **NPC Name:** Display name from Identity component (e.g., "Farmer Alric")
- **Dialogue Text:** Response content from DialogueResponseEvent
- **Icon:** Simple emoji/character indicator (💬 for now, portraits later)

**No MVP:**
- ❌ NPC portrait/avatar (requires assets)
- ❌ Dialogue history (adds complexity)
- ❌ Player response choices (not in scope yet)
- ❌ Mood indicators (can add in S1.16d polish)

### 4. Styling: Minimal & Readable

**Colors:**
- Background: Semi-transparent dark gray (Color::srgba(0.1, 0.1, 0.1, 0.9))
- Border: Light gray (Color::srgb(0.3, 0.3, 0.3))
- NPC Name: Yellow/gold (Color::srgb(1.0, 0.9, 0.4)) - stands out
- Dialogue Text: White (Color::WHITE)
- Icon: White emoji (💬)

**Dimensions:**
- Width: 350px (enough for ~40-50 characters per line)
- Max Height: 200px (allows ~4-5 lines of wrapped text)
- Padding: 12px
- Border: 2px solid
- Position: 20px from bottom-right corner

---

## Architecture Design

### Module Structure

```
src/
  ui/
    mod.rs           # Re-exports UiPlugin
    plugin.rs        # UiPlugin registration
    dialogue_panel/
      mod.rs         # Re-exports components + systems
      components.rs  # DialoguePanel, DialoguePanelSettings
      systems.rs     # spawn_dialogue_panel, update_dialogue_panel, despawn_dialogue_panel

  speech_bubble/     # DEPRECATED - keep for reference but disable
    (existing files, marked deprecated)
```

### Component Design

#### DialoguePanel Component
```rust
#[derive(Component)]
pub struct DialoguePanel {
    npc_id: String,           // Speaker's NPC ID
    speaker_name: String,     // Display name
    content: String,          // Dialogue text
    lifetime: Timer,          // Auto-despawn timer
    fade_start: f32,          // When to start fade (lifetime - fade_seconds)
}
```

#### DialoguePanelSettings Resource
```rust
#[derive(Resource)]
pub struct DialoguePanelSettings {
    pub lifetime_seconds: f32,   // Default: 10.0 (reuse from SpeechBubbleSettings)
    pub fade_seconds: f32,       // Default: 2.0
    pub panel_width: f32,        // Default: 350.0
    pub panel_max_height: f32,   // Default: 200.0
    pub padding: f32,            // Default: 12.0
    pub border_width: f32,       // Default: 2.0
    pub bottom_offset: f32,      // Default: 20.0
    pub right_offset: f32,       // Default: 20.0
}
```

#### DialoguePanelTracker Resource
```rust
#[derive(Resource, Default)]
pub struct DialoguePanelTracker {
    pub active_panel: Option<Entity>,  // Only one panel active at a time
}
```

---

## Implementation Plan (Step-by-Step)

### Phase 1: Module Setup (5 min)

**Steps:**
1. Create `src/ui/` directory
2. Create `src/ui/mod.rs` with module exports
3. Create `src/ui/plugin.rs` with UiPlugin skeleton
4. Create `src/ui/dialogue_panel/` directory
5. Create component/system files with stubs
6. Update `src/main.rs` imports

**Deliverables:**
- ✅ Directory structure created
- ✅ UiPlugin skeleton compiles
- ✅ No functionality yet (stubs only)

### Phase 2: Component & Resource Definition (5 min)

**Steps:**
1. Define `DialoguePanel` component in `components.rs`
2. Define `DialoguePanelSettings` resource with defaults
3. Define `DialoguePanelTracker` resource
4. Add impl blocks for helper methods (new, tick, fade_alpha, is_finished)

**Deliverables:**
- ✅ All components/resources defined
- ✅ Compiles with no warnings

### Phase 3: Spawn System (10 min)

**Steps:**
1. Create `spawn_dialogue_panel` system in `systems.rs`
2. Listen to `DialogueResponseEvent` (same as speech bubbles)
3. Look up NPC Identity for display name
4. Check `DialoguePanelTracker` - despawn old panel if exists
5. Spawn NodeBundle hierarchy:
   ```
   Root NodeBundle (container)
   ├─ Header NodeBundle (name + icon row)
   │  ├─ Icon Text ("💬")
   │  └─ Name Text ("Farmer Alric")
   └─ Body NodeBundle (dialogue text)
      └─ Dialogue Text (content with wrapping)
   ```
6. Insert `DialoguePanel` component on root
7. Update `DialoguePanelTracker` with new entity

**Deliverables:**
- ✅ Panels spawn when DialogueResponseEvent fires
- ✅ Positioned at bottom-right corner
- ✅ Correct styling (colors, padding, border)
- ✅ Text wrapping works

### Phase 4: Update & Despawn System (10 min)

**Steps:**
1. Create `update_dialogue_panel` system
2. Query for `DialoguePanel` + child `Text` nodes
3. Tick lifetime timer
4. Apply fade-out alpha to all text/background during final seconds
5. Despawn when timer finishes
6. Update `DialoguePanelTracker` when despawning

**Deliverables:**
- ✅ Panels auto-despawn after lifetime
- ✅ Fade-out works smoothly
- ✅ Tracker stays in sync

### Phase 5: Plugin Registration (5 min)

**Steps:**
1. Register systems in `UiPlugin::build()`
   - Startup: None needed
   - Update: `spawn_dialogue_panel`, `update_dialogue_panel`
2. Insert resources: `DialoguePanelSettings::default()`, `DialoguePanelTracker::default()`
3. Update `main.rs` to add `UiPlugin` AFTER `DialoguePlugin`
4. Disable `SpeechBubblePlugin` registration (comment out or remove)

**Deliverables:**
- ✅ UiPlugin registered correctly
- ✅ SpeechBubblePlugin disabled
- ✅ Compiles successfully

### Phase 6: Testing & Polish (10-15 min)

**Steps:**
1. Run `cargo run` and trigger dialogue (trade events)
2. Verify panel appears bottom-right with correct text
3. Verify NPC name shows correctly
4. Verify fade-out works
5. Verify new dialogue replaces old immediately
6. Test with multiple NPCs speaking in sequence
7. Adjust spacing/padding if needed

**Deliverables:**
- ✅ Dialogue displays correctly in UI panel
- ✅ No visual glitches
- ✅ Performance acceptable (should be instant)

---

## Technical Implementation Details

### NodeBundle Hierarchy (Detailed)

```rust
// Root container
commands.spawn((
    Node {
        position_type: PositionType::Absolute,
        bottom: Val::Px(settings.bottom_offset),
        right: Val::Px(settings.right_offset),
        width: Val::Px(settings.panel_width),
        max_height: Val::Px(settings.panel_max_height),
        padding: UiRect::all(Val::Px(settings.padding)),
        border: UiRect::all(Val::Px(settings.border_width)),
        flex_direction: FlexDirection::Column,
        ..default()
    },
    BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.9)),
    BorderColor(Color::srgb(0.3, 0.3, 0.3)),
    DialoguePanel::new(npc_id, speaker_name, content, settings.lifetime_seconds, settings.fade_seconds),
))
.with_children(|parent| {
    // Header row (icon + name)
    parent.spawn(Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        margin: UiRect::bottom(Val::Px(8.0)),
        ..default()
    })
    .with_children(|header| {
        // Icon
        header.spawn((
            Text::new("💬 "),
            TextFont { font_size: 20.0, ..default() },
            TextColor(Color::WHITE),
        ));

        // NPC Name
        header.spawn((
            Text::new(speaker_name),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::srgb(1.0, 0.9, 0.4)), // Yellow/gold
        ));
    });

    // Dialogue text body
    parent.spawn((
        Text::new(content),
        TextFont { font_size: 16.0, ..default() },
        TextColor(Color::WHITE),
        Node {
            max_width: Val::Px(settings.panel_width - settings.padding * 2.0),
            ..default()
        },
    ));
});
```

### Fade-Out Logic

```rust
pub fn update_dialogue_panel(
    mut commands: Commands,
    time: Res<Time>,
    mut tracker: ResMut<DialoguePanelTracker>,
    mut panel_query: Query<(Entity, &mut DialoguePanel, &Children)>,
    mut background_query: Query<&mut BackgroundColor>,
    mut text_query: Query<&mut TextColor>,
) {
    for (entity, mut panel, children) in panel_query.iter_mut() {
        panel.tick(time.delta());

        if panel.is_finished() {
            tracker.active_panel = None;
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Apply fade
        let alpha = panel.fade_alpha();

        // Fade background
        if let Ok(mut bg) = background_query.get_mut(entity) {
            bg.0.set_alpha(alpha * 0.9); // Maintain some transparency
        }

        // Fade all text children
        for &child in children.iter() {
            if let Ok(mut text_color) = text_query.get_mut(child) {
                text_color.0.set_alpha(alpha);
            }
        }
    }
}
```

---

## Migration Strategy

### Handling Transition Period

**Option 1: Clean Break (Recommended)**
- Remove SpeechBubblePlugin registration from `main.rs`
- Keep `src/ui/speech_bubble/` code but add README warning
- All dialogue flows through UiPlugin immediately

**Option 2: Gradual Migration**
- Keep both plugins active temporarily
- Add feature flag to toggle between them
- Remove speech bubble after UI panel proven stable

**Recommendation:** Option 1 (clean break) - simpler, faster, no confusion.

### Code Cleanup

**After UI panel proven stable (S1.16d or later):**
1. Delete `src/ui/speech_bubble/` directory entirely
2. Remove `SpeechBubblePlugin` imports from `main.rs`
3. Update CHANGELOG.md with migration note
4. Update all READMEs mentioning speech bubbles

---

## Testing Checklist

**Before Committing:**
- [ ] `cargo fmt`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo check --all-targets`
- [ ] `cargo run` - dialogue appears in UI panel
- [ ] NPC name displays correctly
- [ ] Text wrapping works for long dialogue
- [ ] Fade-out smooth during final seconds
- [ ] Panel despawns after lifetime
- [ ] Multiple NPCs speaking in sequence works
- [ ] No z-fighting or visual glitches
- [ ] Performance acceptable (no stuttering)

**User Acceptance:**
- [ ] Panel doesn't obstruct NPCs
- [ ] Text is readable against all backgrounds
- [ ] Auto-advance timing feels natural
- [ ] Panel position feels intuitive

---

## Documentation Updates

**Files to Update:**
1. `.agent/tasks.yaml` - Add S1.16c task, mark S1.16a as abandoned
2. `.agent/ai_memory.V.1.yaml` - Already updated with pivot decision
3. `CHANGELOG.md` - Add entry for UI panel system
4. `TASK.md` - Update S1.16 checklist with new approach
5. `README.md` - Update feature list (remove "world-space bubbles", add "UI dialogue panel")
6. `src/ui/dialogue_panel/README.md` - Create new README documenting system
7. `src/ui/speech_bubble/README.md` - Add DEPRECATED warning

---

## Future Enhancements (Out of Scope for S1.16c)

**S1.16d - Dialogue Panel Polish:**
- Dialogue history (scrolling log of last 5 messages)
- NPC portraits/avatars in panel
- Mood indicator (emoji or color border based on dopamine level)
- Sound effects on dialogue appear/disappear
- Panel slide-in/slide-out animations

**S1.17 - Interactive Dialogue:**
- Player dialogue choice buttons (branch conversations)
- Dialogue topics/questions (ask about specific events)
- Trade negotiation UI (haggling buttons)

**M10 - Multiplayer:**
- Chat panel for player-to-player communication
- Separate tabs for NPC vs player dialogue

---

## Success Criteria

**Definition of Done:**
- ✅ Dialogue appears in UI panel at bottom-right corner
- ✅ NPC name + icon + dialogue text all visible
- ✅ Text wrapping works for long messages
- ✅ Auto-despawn after lifetime with smooth fade-out
- ✅ Multiple NPCs can speak in sequence without glitches
- ✅ Code passes `cargo fmt` + `cargo clippy`
- ✅ Documentation updated
- ✅ User confirms it's an improvement over broken world-space bubbles

**Stretch Goal:**
- Panel feels polished and professional
- No visual jarring or clipping
- Timing feels natural (not too fast, not too slow)

---

## Time Estimate Breakdown

| Phase | Description | Estimated Time |
|-------|-------------|----------------|
| 1     | Module setup | 5 min |
| 2     | Components/resources | 5 min |
| 3     | Spawn system | 10 min |
| 4     | Update/despawn | 10 min |
| 5     | Plugin registration | 5 min |
| 6     | Testing & polish | 10-15 min |
| **Total** | | **45-50 min** |

**Buffer:** +10-15 min for unexpected issues
**Total with Buffer:** **55-65 min** (~1 hour)

---

## Risk Assessment

**Low Risk:**
- ✅ Standard Bevy UI (well-documented, battle-tested)
- ✅ Reusing existing DialogueResponseEvent (no event changes)
- ✅ Similar component structure to SpeechBubble (familiar patterns)

**Medium Risk:**
- ⚠️ Text wrapping behavior in NodeBundle (may need tweaking)
- ⚠️ Fade-out alpha application to children (query complexity)

**Mitigation:**
- Test text wrapping with very long dialogue early
- Start with simple solid colors, add fade later if time permits

---

## Questions for User (If Needed)

1. **Panel position:** Bottom-right OK, or prefer bottom-left/center?
2. **Lifetime:** Keep 10 seconds like speech bubbles, or adjust?
3. **NPC icon:** Emoji 💬 for MVP, or skip icon entirely?
4. **Speech bubble code:** Delete immediately or keep as reference?

**Default Assumptions (proceed if no response):**
- Bottom-right panel
- 10 second lifetime
- 💬 emoji icon
- Keep speech bubble code but mark deprecated

---

**READY TO IMPLEMENT?** 🚀

Next step: Execute Phase 1 (Module Setup) after user approval.
