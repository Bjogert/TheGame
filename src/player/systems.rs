//! Systems for player interaction with NPCs.
use crate::{
    dialogue::{
        events::DialogueResponseEvent,
        queue::DialogueRequestQueue,
        types::{DialogueContext, DialogueRequest, DialogueTopicHint},
    },
    npc::components::{Identity, InConversation, NpcId},
    player::components::{
        NearbyNpcInfo, Player, PlayerInteractionState, PlayerResponseButton, PlayerResponseWindow,
    },
};
use bevy::log::{debug, info, warn};
use bevy::prelude::*;

/// Maximum distance (in world units) for player-NPC interaction.
const INTERACTION_RANGE: f32 = 3.0;

/// Canned responses the player can choose from when replying to an NPC.
const PLAYER_RESPONSE_OPTIONS: [&str; 3] = [
    "That's interesting! Tell me more.",
    "How can I help with that?",
    "Sounds tough. Stay strong out there.",
];

/// Detects NPCs near the player and updates interaction state.
#[allow(clippy::type_complexity)]
pub fn detect_nearby_npcs(
    player_query: Query<&Transform, With<Player>>,
    npc_query: Query<(&Transform, &Identity), (With<Identity>, Without<InConversation>)>,
    mut interaction_state: ResMut<PlayerInteractionState>,
) {
    let Ok(player_transform) = player_query.single() else {
        interaction_state.nearby_npc = None;
        return;
    };
    let player_pos = player_transform.translation;

    let mut nearest: Option<(&Identity, f32)> = None;
    for (npc_transform, identity) in npc_query.iter() {
        let distance = player_pos.distance(npc_transform.translation);
        if distance <= INTERACTION_RANGE {
            if let Some((_, best)) = nearest {
                if distance < best {
                    nearest = Some((identity, distance));
                }
            } else {
                nearest = Some((identity, distance));
            }
        }
    }

    interaction_state.nearby_npc = nearest.map(|(identity, distance)| NearbyNpcInfo {
        npc_id: identity.id,
        name: identity.display_name.clone(),
        distance,
    });
}

/// Handles player input to initiate dialogue with nearby NPCs.
pub fn handle_player_interaction_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut interaction_state: ResMut<PlayerInteractionState>,
    mut queue: ResMut<DialogueRequestQueue>,
) {
    if !keyboard.just_pressed(KeyCode::KeyE) {
        return;
    }

    let Some(nearby) = interaction_state.nearby_npc.clone() else {
        debug!("Player pressed E but no NPC nearby");
        return;
    };

    let context = DialogueContext {
        summary: Some(format!(
            "The player initiated a conversation with {}.",
            nearby.name
        )),
        ..Default::default()
    };

    let prompt = format!(
        "{} notices the player nearby and greets them. Respond naturally to the player.",
        nearby.name
    );

    let request = DialogueRequest::new(
        nearby.npc_id,
        Some(NpcId::player()),
        prompt,
        DialogueTopicHint::Status,
        context,
    );

    let request_id = queue.enqueue(request);

    interaction_state.active_dialogue = Some(nearby.npc_id);
    interaction_state.active_npc_name = Some(nearby.name.clone());
    interaction_state.last_npc_line = None;

    info!(
        "Player initiates conversation with {} (distance: {:.1}, request #{})",
        nearby.name,
        nearby.distance,
        request_id.value()
    );
}

/// Spawns (or refreshes) the response window when an NPC addresses the player.
pub fn spawn_player_response_window(
    mut commands: Commands,
    mut interaction_state: ResMut<PlayerInteractionState>,
    mut responses: MessageReader<DialogueResponseEvent>,
    identities: Query<&Identity>,
    children_query: Query<&Children>,
) {
    for event in responses.read() {
        let Some(target) = event.response.target else {
            continue;
        };
        if !target.is_player() {
            continue;
        }

        let npc_id = event.response.speaker;
        let Some(npc_identity) = identities.iter().find(|identity| identity.id == npc_id) else {
            warn!(
                "NPC identity for {} not found when spawning response window",
                npc_id
            );
            continue;
        };

        if let Some(window) = interaction_state.response_window.take() {
            despawn_with_children(&mut commands, window, &children_query);
        }

        interaction_state.active_dialogue = Some(npc_id);
        interaction_state.active_npc_name = Some(npc_identity.display_name.clone());
        interaction_state.last_npc_line = Some(event.response.content.clone());

        let window = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(20.0),
                    left: Val::Px(20.0),
                    width: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    border: UiRect::all(Val::Px(2.0)),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    ..Default::default()
                },
                BackgroundColor(Color::srgba(0.08, 0.08, 0.1, 0.95)),
                BorderColor::from(Color::srgb(0.3, 0.3, 0.32)),
                PlayerResponseWindow,
                Name::new("Player Response Window"),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!(
                        "{} says:\n\"{}\"",
                        npc_identity.display_name, event.response.content
                    )),
                    TextFont {
                        font_size: 16.0,
                        ..Default::default()
                    },
                    TextColor(Color::WHITE),
                ));

                for (index, option) in PLAYER_RESPONSE_OPTIONS.iter().enumerate() {
                    parent
                        .spawn((
                            Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                border: UiRect::all(Val::Px(1.5)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                ..Default::default()
                            },
                            Button,
                            Interaction::None,
                            BackgroundColor(Color::srgba(0.18, 0.18, 0.22, 0.95)),
                            BorderColor::from(Color::srgb(0.4, 0.4, 0.45)),
                            PlayerResponseButton {
                                npc_id,
                                response_index: index,
                            },
                            Name::new(format!("Player Response Button {}", index)),
                        ))
                        .with_children(|button| {
                            button.spawn((
                                Text::new(*option),
                                TextFont {
                                    font_size: 15.0,
                                    ..Default::default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                }
            })
            .id();

        interaction_state.response_window = Some(window);
    }
}

/// Handles button presses in the player response window and queues follow-up dialogue.
#[allow(clippy::type_complexity)]
pub fn handle_player_response_buttons(
    mut commands: Commands,
    mut interaction_state: ResMut<PlayerInteractionState>,
    mut queue: ResMut<DialogueRequestQueue>,
    children_query: Query<&Children>,
    mut buttons: Query<(&Interaction, &PlayerResponseButton), (Changed<Interaction>, With<Button>)>,
) {
    for (interaction, button) in buttons.iter_mut() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        let Some(active_npc) = interaction_state.active_dialogue else {
            continue;
        };
        if active_npc != button.npc_id {
            continue;
        }

        let Some(npc_name) = interaction_state.active_npc_name.as_deref() else {
            continue;
        };

        let player_reply = PLAYER_RESPONSE_OPTIONS
            .get(button.response_index)
            .copied()
            .unwrap_or(PLAYER_RESPONSE_OPTIONS[0]);

        let prompt = interaction_state
            .last_npc_line
            .as_deref()
            .map(|last_line| {
                format!(
                    "{npc_name} previously said: \"{last_line}\". The player replies: \"{player_reply}\". Respond in character to the player's reply.",
                )
            })
            .unwrap_or_else(|| {
                format!(
                    "{npc_name} hears the player say: \"{player_reply}\". Respond in character to the player.",
                )
            });

        let context = DialogueContext {
            summary: Some(format!("Player replies: {}", player_reply)),
            ..Default::default()
        };

        queue.enqueue(DialogueRequest::new(
            active_npc,
            Some(NpcId::player()),
            prompt,
            DialogueTopicHint::Status,
            context,
        ));

        if let Some(window) = interaction_state.response_window.take() {
            despawn_with_children(&mut commands, window, &children_query);
        }

        interaction_state.last_npc_line = None;
    }
}

/// Cleans up the response window when no conversations with the player remain.
pub fn cleanup_player_response_window(
    mut commands: Commands,
    mut interaction_state: ResMut<PlayerInteractionState>,
    conversing: Query<&InConversation>,
    children_query: Query<&Children>,
) {
    if interaction_state.response_window.is_none() {
        return;
    }

    let player_in_conversation = conversing
        .iter()
        .any(|conversation| conversation.partner.is_player());

    if !player_in_conversation {
        if let Some(window) = interaction_state.response_window.take() {
            despawn_with_children(&mut commands, window, &children_query);
        }
        interaction_state.active_dialogue = None;
        interaction_state.active_npc_name = None;
        interaction_state.last_npc_line = None;
    }
}

fn despawn_with_children(
    commands: &mut Commands,
    entity: Entity,
    children_query: &Query<&Children>,
) {
    if let Ok(children) = children_query.get(entity) {
        let child_ids = children.to_vec();
        for child in child_ids {
            despawn_with_children(commands, child, children_query);
        }
    }
    commands.entity(entity).despawn();
}
