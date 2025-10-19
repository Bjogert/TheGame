# Speech Bubble UI Module

Floating dialogue text above NPCs in 3D world space.

## Overview

This module implements speech bubbles that appear above NPCs when they speak. Bubbles automatically:
- Spawn from `DialogueResponseEvent` (listening to the dialogue system)
- Position above NPC heads and follow their movement
- Face the camera (billboard effect)
- Fade out and despawn after a lifetime (10 seconds default)

## Current Features (S1.16a - MVP)

- ✅ **Text2d rendering** - Uses Bevy's built-in Text2d component (no external dependencies)
- ✅ **Event-driven spawning** - Subscribes to `DialogueResponseEvent` from DialoguePlugin
- ✅ **NPC tracking** - Bubbles follow their target NPC's position
- ✅ **Billboard rotation** - Text always faces the camera (Y-axis only, no roll)
- ✅ **Lifetime management** - Bubbles despawn after 10 seconds
- ✅ **Fade-out animation** - Alpha lerps to 0 during final 1.5 seconds

## Planned Features

### S1.16b - Visual Polish
- ⏳ **Distance-based culling** - Hide bubbles beyond `max_distance` for performance
- ⏳ **Improved positioning** - Fine-tune Y-offset above capsule meshes
- ⏳ **Word wrapping** - Keep lines under 40 characters for readability

### S1.16c - Personality & Volume
- ⏳ **Speech volume levels** - Whisper (0.6x), Normal (1.0x), Loud (1.5x) font sizes
- ⏳ **Content-based detection** - CAPS = loud, "whisper" keyword = quiet
- ⏳ **Mood integration** - Depressed NPCs speak quieter, Energised NPCs speak louder
- ⏳ **Per-NPC personality traits** - Shy/Boisterous affecting default volume

## Architecture

### Components

**`SpeechBubble`** - Marker component for bubble entities
- `lifetime: Timer` - Countdown to fade-out start
- `fade_duration: f32` - How long the fade-out animation takes
- `max_distance: f32` - Visibility range (for S1.16b culling)

**`SpeechBubbleTarget`** - Links bubble to NPC entity
- `npc_entity: Entity` - Which NPC to follow
- `y_offset: f32` - Height above NPC origin (default: 2.5 units)

**`SpeechVolume`** (S1.16c) - Volume levels affecting size/distance
- `Whisper` - 18pt font, 15u max distance
- `Normal` - 24pt font, 25u max distance
- `Loud` - 36pt font, 40u max distance

### Systems

1. **`spawn_speech_bubbles`** - Listens to `DialogueResponseEvent`, spawns Text2d entities above speaking NPCs
2. **`update_speech_bubble_positions`** - Tracks NPC GlobalTransform, updates bubble positions
3. **`billboard_speech_bubbles`** - Rotates bubbles to face camera (Y-axis only)
4. **`tick_speech_bubble_lifetimes`** - Advances timers, despawns fully faded bubbles
5. **`fade_speech_bubbles`** - Lerps TextColor alpha during fade-out phase

All systems run in `Update` schedule, chained in sequence for frame-coherent behavior.

## Usage

### Basic Setup

The `SpeechBubblePlugin` is automatically registered in `main.rs` after `DialoguePlugin`:

```rust
App::new()
    .add_plugins((
        DefaultPlugins,
        DialoguePlugin,
        // ... other plugins ...
        SpeechBubblePlugin, // Must come after DialoguePlugin
    ))
    .run();
```

Bubbles spawn automatically when NPCs speak (via `DialogueResponseEvent`). No manual spawning required.

### Customization (S1.16b+)

```rust
// Custom lifetime and fade duration
let bubble = SpeechBubble::with_timing(15.0, 2.0); // 15s lifetime, 2s fade

// Custom max distance for culling
let bubble = SpeechBubble::new().with_max_distance(30.0);

// Custom Y-offset above NPC
let target = SpeechBubbleTarget::with_offset(npc_entity, 3.0);
```

## Design Notes

### Why Text2d Instead of UI Text?

- **World-space positioning**: Text2d entities exist in 3D world space, making it trivial to position them above NPCs
- **No bevy_mod_billboard dependency**: The plugin we found (`bevy_mod_billboard`) is stuck at Bevy 0.14, incompatible with our Bevy 0.17 codebase
- **Manual billboarding is simple**: ~10 lines of code to calculate rotation toward camera
- **Performance**: Bevy 0.17 has improved Text2d glyph batching, making this approach efficient

### Distance-Based Culling Design (S1.16b)

Simulates real-world speech audibility:
- Nearby NPCs: Easy to read
- Distant NPCs: Text naturally smaller (perspective), then culled beyond `max_distance`
- Encourages spatial awareness: Players must approach to "overhear" conversations
- Performance benefit: Don't render unreadable text

### Volume Personality System (S1.16c)

Font size variations create distinct NPC "voices":
- **Whisper** (18pt): Secrets, shy NPCs, depressed mood
- **Normal** (24pt): Casual conversation, content mood
- **Loud** (36pt): Shouting, warnings, energised mood, boisterous personality

Auto-detection heuristics:
- `>50% CAPS characters` → Loud
- Keywords like "whisper", "quietly" → Whisper
- `text.len() < 20` → Whisper (short phrases)
- Mood modifiers: Depressed -20% size, Energised +20% size (multiplicative with base volume)

## Dependencies

- `bevy::prelude` - ECS, Text2d, Transform, Query
- `crate::dialogue::events` - DialogueResponseEvent
- `crate::npc::components` - Identity (to find NPC by speaker ID)
- `crate::world::components` - FlyCamera (for billboard rotation)

## Testing

### Visual Verification

1. Run the game: `cargo run`
2. Press F7 to trigger a dialogue probe
3. Look for white text appearing above an NPC's head
4. Observe:
   - Text faces the camera as you move
   - Text follows the NPC if they move
   - Text fades out after ~10 seconds

### Debug Commands

- **F7** - Enqueue test dialogue request (verifies OpenAI connectivity + speech bubbles)

## Future Work

### Post-S1.16c Enhancements

- **Speech bubble backgrounds** - Rounded rect or cloud shape behind text
- **Tail/pointer** - Visual arrow pointing to NPC's head
- **Color coding** - Yellow (trade), white (casual), red (shouting/warning)
- **Sound effects** - Quiet "murmur" when bubbles spawn
- **Accessibility** - HUD log of nearby speech for players with vision difficulties
- **Multiple lines** - Support for longer dialogue with line breaks
- **Emoji/symbols** - ! for surprise, ? for questions, Z for sleeping

## Performance Considerations

### Current (S1.16a)

- **Text rendering**: Bevy 0.17's improved glyph batching handles 10-20 concurrent bubbles efficiently
- **Billboard rotation**: Simple quaternion math per bubble per frame (~negligible)
- **Lifetime tracking**: Single timer tick per bubble per frame

### Optimizations (S1.16b)

- **Distance culling**: Hide bubbles beyond ~25-40 units (typical visibility range)
- **Despawn on lifetime expiry**: Remove entities promptly after fade completes
- **Limit concurrent bubbles**: Cap at 15-20 visible bubbles (cull oldest if exceeded)

### Estimated Budget

- **10 bubbles**: ~0.1ms per frame (text already batched by Bevy)
- **100+ NPCs talking**: Culling keeps visible bubbles manageable (only nearby NPCs)
- **Text mesh generation**: One-time cost when bubble spawns, then cached by Bevy

## Troubleshooting

### Bubbles Don't Appear

1. Check `DialogueResponseEvent` is being emitted (look for dialogue in logs)
2. Verify `SpeechBubblePlugin` is registered AFTER `DialoguePlugin` in `main.rs`
3. Ensure FiraMono-Medium.ttf font exists in `assets/fonts/` (Bevy default)

### Bubbles Don't Face Camera

1. Verify `FlyCamera` component exists on the camera entity
2. Check `billboard_speech_bubbles` system is running (should log errors if camera missing)

### Performance Issues

1. Count active bubbles: `cargo run --features core_debug` (look for entity counts in logs)
2. Enable distance culling (S1.16b) to limit visible bubbles
3. Reduce `lifetime` duration in `SpeechBubble::new()` to despawn bubbles sooner

## References

- [Bevy Text2d docs](https://docs.rs/bevy/latest/bevy/text/struct.Text2d.html)
- [Billboard rotation math](https://en.wikipedia.org/wiki/Billboard_(computer_graphics))
- Research notes: Claude analysis of bevy_mod_billboard alternatives (2025-10-19)
