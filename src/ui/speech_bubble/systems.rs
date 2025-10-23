// src/ui/speech_bubble/systems.rs
//
// Systems for spawning, updating, and despawning speech bubbles using world-space Text2d.

use bevy::prelude::*;

use crate::dialogue::events::DialogueResponseEvent;
use crate::npc::components::Identity;
use crate::world::components::FlyCamera;

use super::components::{SpeechBubble, SpeechBubbleSettings, SpeechBubbleTracker};

// Visual constants
const TEXT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);

/// Spawn or update speech bubbles when NPCs speak.
///
/// Creates Text2d entities positioned in world space above NPCs.
pub fn spawn_speech_bubbles(
    mut commands: Commands,
    mut tracker: ResMut<SpeechBubbleTracker>,
    settings: Res<SpeechBubbleSettings>,
    mut events: MessageReader<DialogueResponseEvent>,
    npc_query: Query<(Entity, &Identity, &GlobalTransform)>,
) {
    for event in events.read() {
        let npc_id = event.response.speaker;

        // Find the NPC entity and position
        let Some((speaker_entity, identity, npc_transform)) =
            npc_query.iter().find(|(_, id, _)| id.id == npc_id)
        else {
            warn!("Cannot spawn speech bubble: NPC {} not found", npc_id);
            continue;
        };

        let content = event.response.content.clone();

        info!(
            "Spawning speech bubble for {} ({}): \"{}\"",
            npc_id, identity.display_name, content
        );

        // Calculate initial world position above NPC
        let mut world_position = npc_transform.translation();
        world_position.y += settings.vertical_offset;

        // If bubble already exists for this NPC, update it
        if let Some(&bubble_entity) = tracker.by_npc.get(&npc_id) {
            // Reset the bubble with new content and reset timer
            commands
                .entity(bubble_entity)
                .insert(SpeechBubble::new(
                    npc_id,
                    speaker_entity,
                    settings.lifetime_seconds,
                ))
                .insert(Text2d::new(content))
                .insert(Transform::from_translation(world_position));
            continue;
        }

        // Otherwise, spawn a new world-space Text2d bubble
        let bubble_entity = commands
            .spawn((
                Text2d::new(content),
                TextFont {
                    font_size: settings.font_size,
                    ..default()
                },
                TextColor(TEXT_COLOR),
                Transform::from_translation(world_position),
                SpeechBubble::new(npc_id, speaker_entity, settings.lifetime_seconds),
                Visibility::Visible,
            ))
            .id();

        tracker.by_npc.insert(npc_id, bubble_entity);
    }
}

/// Update speech bubble positions to follow NPCs in world space.
///
/// Updates Transform to track NPC 3D position, adds billboard rotation,
/// handles lifetime, fade-out, and distance-based culling.
#[allow(clippy::too_many_arguments)]
pub fn update_speech_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<SpeechBubbleSettings>,
    mut tracker: ResMut<SpeechBubbleTracker>,
    camera_query: Query<&GlobalTransform, With<FlyCamera>>,
    speaker_transforms: Query<&GlobalTransform>,
    mut bubble_query: Query<(
        Entity,
        &mut SpeechBubble,
        &mut Transform,
        &mut TextColor,
        &mut Visibility,
    )>,
) {
    let Ok(camera_transform) = camera_query.single() else {
        return; // No camera, can't position or billboard bubbles
    };

    let camera_pos = camera_transform.translation();
    let max_distance_sq = settings.max_display_distance * settings.max_display_distance;

    for (entity, mut bubble, mut transform, mut text_color, mut visibility) in
        bubble_query.iter_mut()
    {
        // Tick the lifetime timer
        bubble.tick(time.delta());

        // Despawn if lifetime expired
        if bubble.is_finished() {
            tracker.by_npc.remove(&bubble.npc_id());
            commands.entity(entity).despawn();
            continue;
        }

        // Get the NPC's current world position
        let Ok(speaker_transform) = speaker_transforms.get(bubble.speaker()) else {
            // NPC entity no longer exists
            tracker.by_npc.remove(&bubble.npc_id());
            commands.entity(entity).despawn();
            continue;
        };

        // Update bubble position to follow NPC (above their head)
        let mut world_position = speaker_transform.translation();
        world_position.y += settings.vertical_offset;
        transform.translation = world_position;

        // Check distance to camera for culling
        let to_camera = camera_pos - world_position;
        if to_camera.length_squared() > max_distance_sq {
            *visibility = Visibility::Hidden;
            continue;
        } else {
            *visibility = Visibility::Visible;
        }

        // Billboard rotation: make text face the camera (Y-axis only, no roll)
        let to_camera_flat = Vec3::new(to_camera.x, 0.0, to_camera.z);
        if to_camera_flat.length_squared() > 0.001 {
            let forward = to_camera_flat.normalize();
            transform.rotation = Quat::from_rotation_arc(Vec3::NEG_Z, forward);
        }

        // Apply fade-out effect during final seconds
        let alpha = bubble.fade_alpha(settings.fade_seconds);
        text_color.0 = TEXT_COLOR.with_alpha(alpha);
    }
}
