// src/ui/dialogue_panel/systems.rs
//
// Systems for spawning, updating, and despawning dialogue panels.

use bevy::{ecs::message::MessageReader, prelude::*};

use crate::dialogue::events::DialogueResponseEvent;
use crate::npc::components::Identity;

use super::components::{DialoguePanel, DialoguePanelSettings, DialoguePanelTracker};

// Visual constants
const BACKGROUND_COLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.9);
const BORDER_COLOR: Color = Color::srgb(0.3, 0.3, 0.3);
const TEXT_COLOR: Color = Color::WHITE;
const NAME_COLOR: Color = Color::srgb(1.0, 0.9, 0.4); // Yellow/gold
const ICON_TEXT: &str = "💬 ";

/// Spawn or update dialogue panels when NPCs speak.
///
/// Creates UI NodeBundle hierarchy positioned at bottom-right corner.
pub fn spawn_dialogue_panel(
    mut commands: Commands,
    mut tracker: ResMut<DialoguePanelTracker>,
    settings: Res<DialoguePanelSettings>,
    mut events: MessageReader<DialogueResponseEvent>,
    npc_query: Query<&Identity>,
) {
    for event in events.read() {
        let npc_id = event.response.speaker;

        // Find the NPC's display name
        let speaker_name = npc_query
            .iter()
            .find(|identity| identity.id == npc_id)
            .map(|identity| identity.display_name.clone())
            .unwrap_or_else(|| format!("NPC-{}", npc_id));

        // Find the target's display name (if speaking to someone specific)
        let target_name = event.response.target.and_then(|target_id| {
            npc_query
                .iter()
                .find(|identity| identity.id == target_id)
                .map(|identity| identity.display_name.clone())
        });

        let content = event.response.content.clone();

        if let Some(ref target) = target_name {
            info!(
                "Spawning dialogue panel for {} ({} → {}): \"{}\"",
                npc_id, speaker_name, target, content
            );
        } else {
            info!(
                "Spawning dialogue panel for {} ({}): \"{}\"",
                npc_id, speaker_name, content
            );
        }

        // If panel already exists, despawn it first
        if let Some(old_panel) = tracker.active_panel {
            commands.entity(old_panel).despawn();
        }

        // Spawn new panel
        let panel_entity = commands
            .spawn((
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
                BackgroundColor(BACKGROUND_COLOR),
                BorderColor::from(BORDER_COLOR),
                DialoguePanel::new(
                    npc_id,
                    speaker_name.clone(),
                    content.clone(),
                    settings.lifetime_seconds,
                    settings.fade_seconds,
                ),
            ))
            .with_children(|parent| {
                // Header row (icon + name)
                parent
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        margin: UiRect::bottom(Val::Px(8.0)),
                        ..default()
                    })
                    .with_children(|header| {
                        // Icon
                        header.spawn((
                            Text::new(ICON_TEXT),
                            TextFont {
                                font_size: settings.icon_font_size,
                                ..default()
                            },
                            TextColor(TEXT_COLOR),
                        ));

                        // NPC Name (with target if available)
                        let display_text = if let Some(ref target) = target_name {
                            format!("{} → {}", speaker_name, target)
                        } else {
                            speaker_name.clone()
                        };

                        header.spawn((
                            Text::new(display_text),
                            TextFont {
                                font_size: settings.name_font_size,
                                ..default()
                            },
                            TextColor(NAME_COLOR),
                        ));
                    });

                // Dialogue text body
                parent.spawn((
                    Text::new(&content),
                    TextFont {
                        font_size: settings.text_font_size,
                        ..default()
                    },
                    TextColor(TEXT_COLOR),
                    Node {
                        max_width: Val::Px(settings.panel_width - settings.padding * 2.0),
                        ..default()
                    },
                ));
            })
            .id();

        tracker.active_panel = Some(panel_entity);
        tracker.by_npc.insert(npc_id, panel_entity);
    }
}

/// Update dialogue panels: tick lifetime, apply fade-out, despawn when finished.
pub fn update_dialogue_panel(
    mut commands: Commands,
    time: Res<Time>,
    mut tracker: ResMut<DialoguePanelTracker>,
    mut panel_query: Query<(Entity, &mut DialoguePanel)>,
    mut background_query: Query<&mut BackgroundColor>,
) {
    for (entity, mut panel) in panel_query.iter_mut() {
        panel.tick(time.delta());

        if panel.is_finished() {
            // Despawn panel
            tracker.active_panel = None;
            tracker.by_npc.remove(&panel.npc_id());
            commands.entity(entity).despawn();
            continue;
        }

        // Apply fade-out during final seconds
        let alpha = panel.fade_alpha();

        // Fade background (maintain transparency)
        if let Ok(mut bg) = background_query.get_mut(entity) {
            bg.0 = BACKGROUND_COLOR.with_alpha(alpha * 0.9);
        }

        // Fade all text children
        // Note: In Bevy 0.17, we query text entities separately since we can't
        // easily traverse descendants. Text entities will fade naturally as panel fades.
        // For now, we just fade the background - text will remain visible.
    }
}
