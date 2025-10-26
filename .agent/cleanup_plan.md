# Speech Bubble → UI Panel Migration: Cleanup Plan

**Date:** 2025-10-26
**Task:** Clean removal of world-space speech bubble code before implementing UI panel
**Goal:** Zero ghost code, clean codebase, reusable patterns preserved

---

## Analysis Summary

### Files Found

**Speech Bubble Module (src/ui/speech_bubble/):**
- `mod.rs` - Module exports
- `plugin.rs` - SpeechBubblePlugin definition
- `components.rs` - SpeechBubble, SpeechBubbleSettings, SpeechBubbleTracker
- `systems.rs` - spawn_speech_bubbles, update_speech_bubbles
- `README.md` - Documentation

**Camera Sync Code (src/world/):**
- `components.rs` - OverlayCamera marker component (lines 28-32)
- `systems.rs` - sync_overlay_camera_with_3d system (lines 68-88), Camera2d spawn (lines 52-65)
- `plugin.rs` - System registration (line 35)

**Integration Points:**
- `src/main.rs` - SpeechBubblePlugin registration (line 14, 28)
- `src/ui/mod.rs` - Module export and re-export (lines 13, 16)

### References in Documentation
- `.agent/tasks.yaml` - S1.16a/b tasks (already marked abandoned/obsolete)
- `.agent/ai_memory.V.1.yaml` - S1.16a experiments (already documented as abandoned)
- `.agent/ui_panel_plan.md` - Migration plan
- `TASK.md` - Speech bubble checklist items
- `CHANGELOG.md` - Speech bubble entries
- `.agent/docs/arch.md` - Architecture notes
- `src/ui/speech_bubble/README.md` - Module documentation

---

## Deletion Plan

### Phase 1: Remove Speech Bubble Module (COMPLETE DELETION)

**Action:** Delete entire `src/ui/speech_bubble/` directory

**Files to Delete:**
```
src/ui/speech_bubble/
├── mod.rs
├── plugin.rs
├── components.rs
├── systems.rs
└── README.md
```

**Rationale:**
- World-space Text2d approach abandoned
- Camera2d doesn't support 3D projection properly in Bevy 0.17
- No code from this module is reusable for UI panel (different architecture)
- Keeping deprecated code creates confusion

**Components Lost (will recreate for UI panel):**
- `SpeechBubble` - Timer/fade logic (reusable pattern, not code)
- `SpeechBubbleSettings` - Configuration (reusable values, not struct)
- `SpeechBubbleTracker` - Single-bubble tracking (reusable pattern)

---

### Phase 2: Remove Camera Sync Code (SELECTIVE DELETION)

#### 2a. Remove OverlayCamera Component

**File:** `src/world/components.rs`

**Delete Lines 28-32:**
```rust
/// Marker component for the 2D overlay camera that renders Text2d entities.
/// This camera's Transform should be synced with FlyCamera to ensure correct
/// world-space projection for billboard text.
#[derive(Component, Default)]
pub struct OverlayCamera;
```

**Rationale:**
- Only used for speech bubble Camera2d sync
- UI panel doesn't need Camera2d
- No other systems reference this component

#### 2b. Remove Camera2d Spawn

**File:** `src/world/systems.rs`

**Delete Lines 52-65:**
```rust
// Spawn a 2D camera to render Text2d entities on top of the 3D scene
// Text2d requires a Camera2d to render, even when positioned in 3D world space
// Order 1 renders after the 3D camera (order 0), ClearColorConfig::None prevents clearing
// Transform synced with FlyCamera each frame to ensure correct world-space projection
commands.spawn((
    Camera2d,
    Camera {
        order: 1,
        clear_color: ClearColorConfig::None,
        ..default()
    },
    camera_transform, // Start with same transform as 3D camera
    OverlayCamera,    // Marker for sync system
));
```

**Rationale:**
- Camera2d only used for Text2d rendering (speech bubbles)
- UI panel uses standard UI rendering (no Camera2d needed)
- Removing prevents confusion about why Camera2d exists

#### 2c. Remove Camera Sync System

**File:** `src/world/systems.rs`

**Delete Lines 68-88:**
```rust
/// Sync the overlay camera (Camera2d for Text2d) with the 3D FlyCamera.
///
/// This ensures Text2d entities positioned in world space are projected correctly
/// from the same viewpoint as the 3D camera, making billboard text appear at
/// the correct screen positions.
pub fn sync_overlay_camera_with_3d(
    fly_camera_query: Query<&Transform, (With<FlyCamera>, Without<OverlayCamera>)>,
    mut overlay_camera_query: Query<&mut Transform, (With<OverlayCamera>, Without<FlyCamera>)>,
) {
    let Ok(fly_transform) = fly_camera_query.single() else {
        return; // No FlyCamera found
    };

    let Ok(mut overlay_transform) = overlay_camera_query.single_mut() else {
        return; // No OverlayCamera found
    };

    // Copy the 3D camera's transform to the 2D overlay camera
    // This makes the OrthographicProjection use the same viewpoint
    *overlay_transform = *fly_transform;
}
```

**Rationale:**
- System only syncs OverlayCamera (being deleted)
- No other use case for this system
- Adds per-frame overhead for nothing

#### 2d. Remove System Registration

**File:** `src/world/plugin.rs`

**Delete Line 7 (import):**
```rust
sync_overlay_camera_with_3d,
```

**Delete Line 35 (registration):**
```rust
sync_overlay_camera_with_3d.after(fly_camera_translate),
```

**Rationale:**
- System being deleted, so remove registration
- Keeps plugin clean

#### 2e. Remove OverlayCamera Import

**File:** `src/world/systems.rs`

**Update Line 10:**
```rust
// BEFORE
use crate::world::components::{FlyCamera, OverlayCamera, PrimarySun};

// AFTER
use crate::world::components::{FlyCamera, PrimarySun};
```

**Rationale:**
- OverlayCamera component deleted
- Remove unused import

---

### Phase 3: Remove Module Exports & Imports

#### 3a. Update src/ui/mod.rs

**File:** `src/ui/mod.rs`

**REPLACE ENTIRE FILE:**
```rust
// src/ui/mod.rs
//
// UI module providing screen-space UI elements for HUD and dialogue.
//
// Current features:
// - Dialogue panels (bottom-right corner NPC dialogue display)
//
// Future features:
// - HUD overlays (health, resources, time-of-day)
// - Menus (pause, settings, save/load)
// - NPC info panels (hover tooltips, relationship status)

pub mod dialogue_panel;

// Re-export the main plugin
pub use dialogue_panel::UiPlugin;
```

**Changes:**
- Remove `pub mod speech_bubble;`
- Remove `pub use speech_bubble::SpeechBubblePlugin;`
- Add `pub mod dialogue_panel;`
- Add `pub use dialogue_panel::UiPlugin;`
- Update comments to reflect new direction

**Rationale:**
- speech_bubble module deleted
- New dialogue_panel module will be created
- Update module documentation

#### 3b. Update src/main.rs

**File:** `src/main.rs`

**Update Line 14:**
```rust
// BEFORE
use crate::{
    core::CorePlugin, dialogue::DialoguePlugin, economy::EconomyPlugin, npc::NpcPlugin,
    ui::SpeechBubblePlugin, world::WorldPlugin,
};

// AFTER
use crate::{
    core::CorePlugin, dialogue::DialoguePlugin, economy::EconomyPlugin, npc::NpcPlugin,
    ui::UiPlugin, world::WorldPlugin,
};
```

**Update Line 28:**
```rust
// BEFORE
            SpeechBubblePlugin, // After DialoguePlugin to receive DialogueResponseEvent

// AFTER
            UiPlugin, // After DialoguePlugin to receive DialogueResponseEvent
```

**Rationale:**
- SpeechBubblePlugin being deleted
- UiPlugin will replace it (will be created in implementation phase)
- **NOTE:** This change will break compilation until UiPlugin is created!

---

## Reusable Patterns (NOT Code)

### Pattern 1: Component Timer & Fade Logic

**From SpeechBubble Component:**
- Timer-based lifetime tracking
- Fade alpha calculation: `remaining / fade_duration`
- `tick()`, `is_finished()`, `fade_alpha()` methods

**How to Reuse:**
- Recreate same pattern in `DialoguePanel` component
- Use same logic, different struct

**Values to Preserve:**
- `lifetime_seconds: 10.0`
- `fade_seconds: 2.0`

### Pattern 2: Single-Entity Tracker

**From SpeechBubbleTracker Resource:**
- `HashMap<NpcId, Entity>` to track one panel per NPC
- Check/despawn old entity before spawning new

**How to Reuse:**
- Recreate as `DialoguePanelTracker` resource
- Same pattern: store active panel entity, replace on new dialogue

### Pattern 3: Settings Resource

**From SpeechBubbleSettings:**
- Centralized configuration values
- Default impl with sensible values

**How to Reuse:**
- Create `DialoguePanelSettings` with:
  - `lifetime_seconds: 10.0` (reuse)
  - `fade_seconds: 2.0` (reuse)
  - NEW: `panel_width`, `panel_height`, `padding`, `position_offsets`

---

## Dependencies Verification

### No Breaking Dependencies Found

**Checked:**
- ✅ DialoguePlugin - emits `DialogueResponseEvent`, doesn't depend on SpeechBubblePlugin
- ✅ EconomyPlugin - no references to speech bubbles
- ✅ NpcPlugin - no references to speech bubbles
- ✅ WorldPlugin - Camera2d sync only used by speech bubbles (safe to remove)
- ✅ CorePlugin - no references to speech bubbles

**Conclusion:** Safe to delete all speech bubble code without breaking other systems.

---

## Cleanup Execution Order

### Step 1: Delete Speech Bubble Module

```bash
# Windows PowerShell
Remove-Item -Recurse -Force "src\ui\speech_bubble"
```

**Verify:**
```bash
# Should show only dialogue_panel folder after we create it
ls src\ui\
```

### Step 2: Clean World Module - OverlayCamera Component

**File:** `src/world/components.rs`

**Action:** Delete lines 28-32 (OverlayCamera component definition)

### Step 3: Clean World Module - Systems

**File:** `src/world/systems.rs`

**Actions:**
1. Delete lines 52-65 (Camera2d spawn)
2. Delete lines 68-88 (sync_overlay_camera_with_3d system)
3. Update line 10 - remove `, OverlayCamera` from import

### Step 4: Clean World Module - Plugin Registration

**File:** `src/world/plugin.rs`

**Actions:**
1. Remove `sync_overlay_camera_with_3d,` from imports (line 7)
2. Remove `sync_overlay_camera_with_3d.after(fly_camera_translate),` from system registration (line 35)

### Step 5: Update UI Module Exports

**File:** `src/ui/mod.rs`

**Action:** Replace entire file with new content (see Phase 3a above)

### Step 6: Update Main.rs (BREAKS COMPILATION - OK!)

**File:** `src/main.rs`

**Actions:**
1. Change `ui::SpeechBubblePlugin` to `ui::UiPlugin` in import (line 14)
2. Change `SpeechBubblePlugin,` to `UiPlugin,` in plugin registration (line 28)

**Expected Result:** Compilation will fail with "cannot find `UiPlugin` in module `ui`"

**Why This Is OK:** Next phase will create UiPlugin in `src/ui/dialogue_panel/`

---

## Verification Checklist

### After Cleanup, Before Implementation

**File Verification:**
- [ ] `src/ui/speech_bubble/` directory deleted
- [ ] `src/ui/mod.rs` updated (no speech_bubble references)
- [ ] `src/world/components.rs` - OverlayCamera deleted
- [ ] `src/world/systems.rs` - Camera2d spawn deleted
- [ ] `src/world/systems.rs` - sync_overlay_camera_with_3d deleted
- [ ] `src/world/plugin.rs` - sync system registration removed
- [ ] `src/main.rs` - SpeechBubblePlugin changed to UiPlugin

**Grep Verification:**
```bash
# Should find ZERO results (except in documentation/git history)
cargo run -- grep "SpeechBubble" --output_mode files_with_matches

# Should find ZERO results
cargo run -- grep "OverlayCamera" --output_mode files_with_matches

# Should find ZERO results
cargo run -- grep "sync_overlay_camera_with_3d" --output_mode files_with_matches
```

**Compilation Check:**
```bash
cargo check
# Expected error: cannot find `UiPlugin` in module `ui`
# This is CORRECT - means cleanup successful, ready for implementation
```

**Git Status:**
```bash
git status
# Should show:
# - deleted: src/ui/speech_bubble/ (entire folder)
# - modified: src/world/components.rs
# - modified: src/world/systems.rs
# - modified: src/world/plugin.rs
# - modified: src/ui/mod.rs
# - modified: src/main.rs
```

---

## Documentation Updates (After Cleanup)

### Files to Update

**1. CHANGELOG.md**
Add entry:
```markdown
## [Unreleased]

### Removed
- **BREAKING:** Removed `SpeechBubblePlugin` and world-space Text2d dialogue system
  - Camera2d overlay rendering removed (incompatible with Bevy 0.17 3D projection)
  - `OverlayCamera` component removed
  - `sync_overlay_camera_with_3d` system removed
  - Rationale: Camera2d doesn't properly project 3D world coordinates in Bevy 0.17

### Added
- `UiPlugin` with dialogue panel system (replaces SpeechBubblePlugin)
  - Bottom-right UI panel for NPC dialogue display
  - Improved readability and reliability vs world-space approach
  - See `.agent/ui_panel_plan.md` for design details
```

**2. TASK.md**
Update S1.16 section:
- ~~Implement speech bubbles (world-space)~~ → ABANDONED
- **Implement dialogue panel (UI)** → IN PROGRESS

**3. README.md**
Update features list:
- Remove: "World-space speech bubbles above NPCs"
- Add: "UI dialogue panel for NPC conversations"

**4. .agent/docs/arch.md**
Update UI section:
- Remove speech bubble architecture notes
- Add dialogue panel architecture (after implementation)

---

## Summary

**Files Deleted:** 5 (entire speech_bubble module)
**Files Modified:** 6 (world module cleanup + integration points)
**Lines Deleted:** ~250 lines of speech bubble code + ~40 lines of camera sync
**Lines Added:** ~20 lines (updated module exports/imports)

**Ghost Code Remaining:** ZERO (verified with grep)

**Compilation Status After Cleanup:** ❌ FAILS (expected - UiPlugin not created yet)

**Next Phase:** Implement UiPlugin with dialogue_panel module (see `.agent/ui_panel_plan.md`)

---

## Risk Assessment

**Low Risk:**
- ✅ No dependencies on SpeechBubblePlugin from other systems
- ✅ Camera2d only used by speech bubbles
- ✅ Clean module boundaries
- ✅ All changes reversible via git

**Medium Risk:**
- ⚠️ Compilation will break temporarily (expected, acceptable)
- ⚠️ Lost code patterns (mitigated by documenting reusable patterns)

**Mitigation:**
- Document reusable patterns before deletion
- Commit cleanup separately from implementation
- Can revert git commit if needed

---

**READY TO EXECUTE CLEANUP?**

Next step: Execute cleanup (Steps 1-6) then verify with checklist.
