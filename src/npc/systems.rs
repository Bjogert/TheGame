//! Systems related to NPC spawning and scheduling.
use bevy::{math::primitives::Capsule3d, prelude::*};

use crate::{
    core::plugin::SimulationClock,
    npc::components::{
        DailySchedule, Identity, LocomotionState, MovementTarget, NpcIdGenerator, NpcLocomotion,
        ScheduleEntry, ScheduleState, ScheduleTicker,
    },
    world::time::WorldClock,
};

/// Spawns a handful of debug NPCs with unique identities.
pub fn spawn_debug_npcs(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut id_generator: ResMut<NpcIdGenerator>,
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
    mut movers: Query<(&Identity, &mut Transform, &mut NpcLocomotion)>,
    world_transforms: Query<&GlobalTransform>,
) {
    let delta_seconds = sim_clock.last_scaled_delta().as_secs_f32();
    if delta_seconds <= f32::EPSILON {
        return;
    }

    for (identity, mut transform, mut locomotion) in movers.iter_mut() {
        let Some(target) = locomotion.target() else {
            continue;
        };

        let target_position = match target {
            MovementTarget::Entity(entity) => match world_transforms.get(entity) {
                Ok(global) => {
                    let mut pos: Vec3 = global.translation().into();
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
