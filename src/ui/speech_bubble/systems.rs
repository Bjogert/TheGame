// src/ui/speech_bubble/systems.rs
//
// Systems for spawning, updating, and despawning speech bubbles using UI nodes.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::dialogue::events::DialogueResponseEvent;
use crate::npc::components::Identity;
use crate::world::components::FlyCamera;

use super::components::{
    SpeechBubble, SpeechBubbleSettings, SpeechBubbleTracker, SpeechBubbleUiRoot,
};

// Visual constants
const BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.85);
const TEXT_COLOR: Color = Color::srgb(1.0, 1.0, 1.0);
const MAX_WIDTH_PX: f32 = 225.0; // 25% smaller than original 300px
const PADDING_PX: f32 = 6.0; // 25% smaller than original 8px

/// Set up the UI root node that holds all speech bubbles.
///
/// This creates a full-screen transparent overlay that speech bubbles
/// are parented to, ensuring they render on top of the 3D scene.
pub fn setup_speech_bubble_root(mut commands: Commands) {
    let root = commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .insert(ZIndex(100)) // Render on top of other UI
        .insert(BackgroundColor(Color::NONE))
        .id();

    commands.insert_resource(SpeechBubbleUiRoot(root));
    info!("Speech bubble UI root created");
}

/// Spawn or update speech bubbles when NPCs speak.
///
/// Listens to `DialogueResponseEvent` and creates UI node entities
/// positioned in screen space that track the 3D position of NPCs.
pub fn spawn_speech_bubbles(
    mut commands: Commands,
    mut tracker: ResMut<SpeechBubbleTracker>,
    settings: Res<SpeechBubbleSettings>,
    mut events: MessageReader<DialogueResponseEvent>,
    npc_query: Query<(Entity, &Identity)>,
    root: Res<SpeechBubbleUiRoot>,
) {
    for event in events.read() {
        let npc_id = event.response.speaker;

        // Find the NPC entity
        let Some((speaker_entity, identity)) = npc_query.iter().find(|(_, id)| id.id == npc_id)
        else {
            warn!("Cannot spawn speech bubble: NPC {} not found", npc_id);
            continue;
        };

        let content = event.response.content.clone();

        info!(
            "Spawning speech bubble for {} ({}): \"{}\"",
            npc_id, identity.display_name, content
        );

        // If bubble already exists for this NPC, update it
        if let Some(&bubble_entity) = tracker.by_npc.get(&npc_id) {
            // Reset the bubble with new content and reset timer
            commands.entity(bubble_entity).insert(SpeechBubble::new(
                npc_id,
                speaker_entity,
                settings.lifetime_seconds,
            ));

            // Update the text
            commands.entity(bubble_entity).insert(Text::new(content));
            continue;
        }

        // Otherwise, spawn a new UI bubble
        let bubble_entity = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    max_width: Val::Px(MAX_WIDTH_PX),
                    padding: UiRect::all(Val::Px(PADDING_PX)),
                    display: Display::None, // Hidden initially, positioned by update system
                    ..default()
                },
                BackgroundColor(BACKGROUND_COLOR),
                ZIndex(101),
                SpeechBubble::new(npc_id, speaker_entity, settings.lifetime_seconds),
                Text::new(content),
                TextFont {
                    font_size: settings.font_size,
                    ..default()
                },
                TextColor(TEXT_COLOR),
            ))
            .id();

        commands.entity(root.0).add_child(bubble_entity);
        tracker.by_npc.insert(npc_id, bubble_entity);
    }
}

/// Update speech bubble positions to follow NPCs in screen space.
///
/// Converts each NPC's 3D world position to 2D screen coordinates and
/// positions the UI bubble accordingly. Also handles lifetime, fade-out,
/// and distance-based culling.
#[allow(clippy::too_many_arguments)] // System function requires all arguments
pub fn update_speech_bubbles(
    mut commands: Commands,
    time: Res<Time>,
    settings: Res<SpeechBubbleSettings>,
    mut tracker: ResMut<SpeechBubbleTracker>,
    camera_query: Query<(&Camera, &GlobalTransform), With<FlyCamera>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    speaker_transforms: Query<&GlobalTransform>,
    mut bubble_query: Query<(
        Entity,
        &mut SpeechBubble,
        &mut Node,
        &mut BackgroundColor,
        &mut TextColor,
    )>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return; // No camera, can't position bubbles
    };

    let Ok(window) = window_query.single() else {
        return; // No window, can't get screen dimensions
    };

    let window_height = window.resolution.height();
    let max_distance_sq = settings.max_display_distance * settings.max_display_distance;

    for (entity, mut bubble, mut style, mut background, mut text_color) in bubble_query.iter_mut() {
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

        // Calculate world position above NPC's head
        let mut world_position = speaker_transform.translation();
        world_position.y += settings.vertical_offset;

        // Check distance to camera
        let to_camera = camera_transform.translation() - world_position;
        if to_camera.length_squared() > max_distance_sq {
            style.display = Display::None;
            continue;
        }

        // Convert world position to viewport (screen) coordinates
        let Ok(viewport_position) = camera.world_to_viewport(camera_transform, world_position)
        else {
            // NPC is behind camera or outside frustum
            style.display = Display::None;
            continue;
        };

        // Position the UI bubble at the screen coordinates
        // UI origin is top-left, so we need to flip Y
        style.display = Display::Flex;
        style.left = Val::Px(viewport_position.x);
        style.top = Val::Px(window_height - viewport_position.y);

        // Apply fade-out effect
        let alpha = bubble.fade_alpha(settings.fade_seconds);
        text_color.0 = TEXT_COLOR.with_alpha(alpha);
        background.0 = BACKGROUND_COLOR.with_alpha(alpha * 0.85);
    }
}
