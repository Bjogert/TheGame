//! Systems related to NPC spawning and scheduling.
use bevy::{ecs::system::ParamSet, math::primitives::Capsule3d, prelude::*};

use crate::{
    core::plugin::SimulationClock,
    dialogue::events::DialogueRequestedEvent,
    npc::components::{
        ConversationState, DailySchedule, Identity, InConversation, LocomotionState,
        MovementTarget, NpcIdGenerator, NpcLocomotion, ScheduleEntry, ScheduleState,
        ScheduleTicker,
    },
    npc::events::NpcActivityChangedEvent,
    npc::motivation::{MotivationConfig, NpcMotivation},
    world::time::WorldClock,
};

/// Spawns a handful of debug NPCs with unique identities.
pub fn spawn_debug_npcs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut id_generator: ResMut<NpcIdGenerator>,
    motivation_config: Res<MotivationConfig>,
) {
    let prototypes = [
        (
            "Alric",
            Color::srgb_u8(200, 90, 90),
            Vec3::new(4.0, 1.0, 2.0),
            vec![
                ScheduleEntry::new(0.00, "Sleeping"),
                ScheduleEntry::new(0.25, "Fetching water"),
                ScheduleEntry::new(0.50, "Working the fields"),
                ScheduleEntry::new(0.75, "Supper & stories"),
            ],
        ),
        (
            "Bryn",
            Color::srgb_u8(90, 150, 210),
            Vec3::new(6.5, 1.0, -1.5),
            vec![
                ScheduleEntry::new(0.00, "Sleeping"),
                ScheduleEntry::new(0.30, "Preparing meals"),
                ScheduleEntry::new(0.55, "Market errands"),
                ScheduleEntry::new(0.80, "Evening lute practice"),
            ],
        ),
        (
            "Cedric",
            Color::srgb_u8(140, 200, 120),
            Vec3::new(3.0, 1.0, -4.0),
            vec![
                ScheduleEntry::new(0.00, "Sleeping"),
                ScheduleEntry::new(0.20, "Tending livestock"),
                ScheduleEntry::new(0.60, "Guard patrol"),
                ScheduleEntry::new(0.85, "Tavern chatter"),
            ],
        ),
    ];

    for (name, color, position, schedule_entries) in prototypes {
        let id = id_generator.next_id();
        let identity = Identity::new(id, name, 24.0);

        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Capsule3d::new(0.3, 1.0)))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: color,
                ..default()
            })),
            Transform::from_translation(position),
            identity,
            DailySchedule::new(schedule_entries),
            ScheduleState::default(),
            NpcLocomotion::default(),
            NpcMotivation::new(&motivation_config),
            Name::new(format!("{} ({})", name, id)),
        ));
    }
}

/// Updates each NPC's current activity when pending ticks exist.
pub fn tick_schedule_state(
    mut ticker: ResMut<ScheduleTicker>,
    sim_clock: Res<SimulationClock>,
    clock: Res<WorldClock>,
    mut query: Query<(&Identity, &DailySchedule, &mut ScheduleState)>,
    mut activity_events: MessageWriter<NpcActivityChangedEvent>,
) {
    let delta = sim_clock.last_scaled_delta().as_secs_f32();
    ticker.accumulate(delta);

    let pending = ticker.take_pending();
    if pending == 0 || query.is_empty() {
        return;
    }

    let time_of_day = clock.time_of_day();

    for (identity, schedule, mut state) in query.iter_mut() {
        if schedule.entries.is_empty() {
            continue;
        }

        let current_activity = current_activity(schedule, time_of_day);
        if state.current_activity != current_activity {
            info!(
                "{} transitions to activity: {}",
                identity.display_name, current_activity
            );
            state.current_activity = current_activity.to_string();
            activity_events.write(NpcActivityChangedEvent {
                npc: identity.id,
                activity: current_activity.to_string(),
                time_of_day,
            });
        }
    }
}

fn current_activity(schedule: &DailySchedule, time_of_day: f32) -> &str {
    let entries = &schedule.entries;
    if entries.len() == 1 {
        return &entries[0].activity;
    }

    let mut selected = &entries[entries.len() - 1];
    for entry in entries {
        if time_of_day >= entry.start {
            selected = entry;
        } else {
            break;
        }
    }

    if time_of_day < entries[0].start {
        selected = &entries[entries.len() - 1];
    }

    selected.activity.as_str()
}

/// Moves NPCs toward their active destinations using the simulation clock delta.
pub fn drive_npc_locomotion(
    sim_clock: Res<SimulationClock>,
    mut movers: Query<(
        &Identity,
        &mut Transform,
        &mut NpcLocomotion,
        Option<&InConversation>,
    )>,
    world_transforms: Query<&GlobalTransform>,
) {
    let delta_seconds = sim_clock.last_scaled_delta().as_secs_f32();
    if delta_seconds <= f32::EPSILON {
        return;
    }

    for (identity, mut transform, mut locomotion, conversation) in movers.iter_mut() {
        // Freeze movement if in conversation (but allow Approaching state)
        if let Some(conv) = conversation {
            if conv.state != ConversationState::Approaching {
                continue; // Skip movement for waiting/speaking NPCs
            }
        }

        let Some(target) = locomotion.target() else {
            continue;
        };

        let target_position = match target {
            MovementTarget::Entity(entity) => match world_transforms.get(entity) {
                Ok(global) => {
                    let mut pos = global.translation();
                    pos.y = transform.translation.y;
                    pos
                }
                Err(_) => {
                    warn!(
                        "Clearing locomotion target for {}: entity {entity:?} missing transform",
                        identity.display_name
                    );
                    locomotion.clear_target();
                    continue;
                }
            },
        };

        let displacement = Vec2::new(
            target_position.x - transform.translation.x,
            target_position.z - transform.translation.z,
        );
        let distance = displacement.length();
        let arrive_distance = locomotion.arrive_distance();

        let was_moving = locomotion.state() == LocomotionState::Moving;

        if distance <= arrive_distance {
            let arrival_label = locomotion.active_label().map(|label| label.to_string());
            transform.translation.x = target_position.x;
            transform.translation.z = target_position.z;
            locomotion.clear_target();

            if was_moving {
                if let Some(label) = arrival_label {
                    info!("{} arrived at {}", identity.display_name, label);
                } else {
                    info!("{} completed travel", identity.display_name);
                }
            }
            continue;
        }

        let direction = displacement / distance;
        let step = locomotion.move_speed() * delta_seconds;
        let travel = direction * step.min(distance);

        transform.translation.x += travel.x;
        transform.translation.z += travel.y;
    }
}

/// Makes NPCs face their conversation partner when stopped during dialogue.
#[allow(clippy::type_complexity)]
pub fn orient_conversing_npcs(
    time: Res<Time>,
    all_identities: Query<(Entity, &Identity)>,
    mut transforms: ParamSet<(
        Query<(Entity, &Identity, &mut Transform, &InConversation)>,
        Query<&Transform>,
    )>,
) {
    // First pass: collect data about who needs to face whom
    let mut rotation_data = Vec::new();

    for (entity, identity, transform, conversation) in transforms.p0().iter() {
        // Only orient when stopped (not while approaching)
        if conversation.state == ConversationState::Approaching {
            continue;
        }

        // Find partner entity
        let Some(partner_entity) = all_identities
            .iter()
            .find(|(_, id)| id.id == conversation.partner)
            .map(|(e, _)| e)
        else {
            warn!(
                "{} in conversation but partner {} not found",
                identity.display_name, conversation.partner
            );
            continue;
        };

        // Store the data for second pass
        rotation_data.push((entity, transform.translation, partner_entity));
    }

    // Second pass: get partner positions and calculate rotations
    for (entity, my_position, partner_entity) in rotation_data {
        // Get partner position (copy it immediately to avoid borrow issues)
        let partner_position = match transforms.p1().get(partner_entity) {
            Ok(partner_transform) => partner_transform.translation,
            Err(_) => continue,
        };

        // Calculate look direction (Y-axis rotation only, no vertical tilt)
        let direction = Vec3::new(
            partner_position.x - my_position.x,
            0.0, // No vertical look
            partner_position.z - my_position.z,
        );

        if direction.length() < 0.01 {
            continue; // Too close, skip rotation
        }

        let direction = direction.normalize();

        // Calculate target rotation (rotate around Y axis to face direction)
        // atan2(x, z) gives angle from forward (-Z) axis
        let angle = direction.x.atan2(-direction.z);
        let target_rotation = Quat::from_rotation_y(angle);

        // Third pass: apply rotation
        if let Ok((_entity, _identity, mut transform, _conversation)) =
            transforms.p0().get_mut(entity)
        {
            transform.rotation = transform
                .rotation
                .slerp(target_rotation, 5.0 * time.delta_secs());
        }
    }
}

/// Starts conversations by adding InConversation components when dialogue is requested.
pub fn start_conversations(
    mut commands: Commands,
    mut events: MessageReader<DialogueRequestedEvent>,
    world_clock: Res<WorldClock>,
    npcs: Query<(Entity, &Identity)>,
) {
    for event in events.read() {
        let Some(target) = event.target else {
            continue; // No conversation if no target
        };

        // Find entities for speaker and target
        let speaker_entity = npcs
            .iter()
            .find(|(_, id)| id.id == event.speaker)
            .map(|(e, _)| e);
        let target_entity = npcs.iter().find(|(_, id)| id.id == target).map(|(e, _)| e);

        if let (Some(speaker_e), Some(target_e)) = (speaker_entity, target_entity) {
            let current_time = world_clock.time_of_day();

            // Add InConversation to speaker
            commands.entity(speaker_e).insert(InConversation::new(
                target,
                event.request_id,
                current_time,
                ConversationState::WaitingAtDestination,
            ));

            // Add InConversation to target
            commands.entity(target_e).insert(InConversation::new(
                event.speaker,
                event.request_id,
                current_time,
                ConversationState::WaitingAtDestination,
            ));

            info!(
                "Started conversation: {} <-> {} (request {})",
                event.speaker,
                target,
                event.request_id.value()
            );
        }
    }
}

/// Cleans up conversations after a timeout period.
/// This removes InConversation components so NPCs can resume their tasks.
pub fn cleanup_conversations(
    mut commands: Commands,
    world_clock: Res<WorldClock>,
    conversing: Query<(Entity, &Identity, &InConversation)>,
) {
    // Conversation timeout in fractional day units (with 10-minute day, 0.01 ≈ 6 seconds)
    const CONVERSATION_TIMEOUT: f32 = 0.013; // ~8 seconds with 10-minute day

    let current_time = world_clock.time_of_day();

    for (entity, identity, conversation) in conversing.iter() {
        let mut elapsed = current_time - conversation.started_at;

        // Handle day wraparound (if conversation started at 0.99 and current is 0.01)
        if elapsed < 0.0 {
            elapsed += 1.0;
        }

        if elapsed >= CONVERSATION_TIMEOUT {
            commands.entity(entity).remove::<InConversation>();
            info!(
                "{} conversation ended (elapsed: {:.3}), resuming activity",
                identity.display_name, elapsed
            );
        }
    }
}
