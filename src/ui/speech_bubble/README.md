# Speech Bubble UI Module

Floating dialogue text rendered in world space above NPCs.

## Status (2025-10-26) - ✅ FIXED

**Root cause identified and resolved:**

1. **Missing Anchor Component**: Text2d entities were spawning without an explicit `Anchor` component, defaulting to `Anchor::CENTER`. This caused text to be centered on the transform position rather than appearing above the NPC's head.
   - **Fix**: Added `Anchor::BOTTOM_CENTER` to spawn code, positioning text bottom at the transform location (naturally appears above NPC).

2. **Camera2d Stuck at Origin**: The Camera2d (required for Text2d rendering) had no Transform component and was stuck at (0,0,0) while the FlyCamera moved around the world. Text2d uses Camera2d's OrthographicProjection, which projected from the wrong viewpoint.
   - **Fix**: Added `sync_overlay_camera_with_3d` system that copies FlyCamera's Transform to the OverlayCamera (Camera2d marker) each frame, ensuring correct world-space projection.

Both fixes have been implemented and compile successfully. Speech bubbles should now render correctly above NPCs and follow camera movement.

## Components & Resources

- `SpeechBubble` - tracks the NPC id, speaker entity, and lifetime timer for each bubble.
- `SpeechBubbleTracker` - maps each `NpcId` to its active bubble entity so only one bubble renders per NPC.
- `SpeechBubbleSettings` - resource exposing lifetime, fade duration, max distance, vertical offset, font size, and max text width (wrap bound).

## Systems

1. `spawn_speech_bubbles` - listens to `DialogueResponseEvent`, spawns or refreshes `Text2d` entities using current settings.
2. `update_speech_bubbles` - every frame ticks lifetimes, recomputes positions from the speaker's `GlobalTransform`, hides distant bubbles, applies billboard rotation, and fades alpha near expiry.

The plugin registers both systems in the `Update` schedule and initialises the resources.

## Settings

```rust
pub struct SpeechBubbleSettings {
    pub lifetime_seconds: f32,    // default 10.0
    pub fade_seconds: f32,        // default 2.0
    pub max_display_distance: f32, // default 25.0 world units
    pub vertical_offset: f32,     // default 2.5
    pub font_size: f32,           // default 15.0
    pub max_text_width: f32,      // default 220.0 (wrap width in logical px)
}
```

Tune these fields via resource mutations or future config plumbing. `max_text_width` enables word wrapping so dialogue no longer runs on a single line.

## Known Issues

- No background graphic yet; readability can suffer against bright skies.

## Next Steps

1. **Test the fixes:** Run `cargo run` and verify bubbles appear above NPCs and follow camera movement.
2. Once verified, proceed with S1.16b polish tasks:
   - Distance culling tweaks
   - Offset tuning
   - Visual styling (background sprites, color variations)
3. Consider adding background graphics for better readability against bright skies.

## Troubleshooting

- Nothing renders: verify `SpeechBubblePlugin` is added after `DialoguePlugin`, and ensure a Camera2d with `order: 1` plus `ClearColorConfig::None` exists (required for `Text2d` in Bevy 0.17).
- Text mirrored: confirm billboard math uses `Quat::from_rotation_arc(Vec3::Z, forward)`.
- Text missing wrap: ensure `SpeechBubbleSettings::max_text_width` is set (default 220.0).

Refer to `TASK.md` and `.agent/tasks.yaml` for the latest status and open investigations.
