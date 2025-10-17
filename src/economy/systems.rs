//! Systems powering the placeholder micro trade loop.
use bevy::{math::primitives::Cuboid, prelude::*};

use crate::{
    dialogue::{
        queue::DialogueRequestQueue,
        types::{
            DialogueContext, DialogueContextEvent, DialogueRequest, DialogueTopicHint,
            TradeContext, TradeContextReason, TradeDescriptor,
        },
    },
    npc::components::{Identity, MovementTarget, NpcId, NpcLocomotion},
    world::time::WorldClock,
};

use super::{
    components::{Inventory, Profession, ProfessionCrate, TradeGood},
    dependency::EconomyDependencyMatrix,
    events::{ProfessionDependencyUpdateEvent, TradeCompletedEvent, TradeReason},
    resources::{MicroTradeLoopState, ProfessionCrateRegistry},
};

const FARMER_NAME: &str = "Alric";
const MILLER_NAME: &str = "Bryn";
const BLACKSMITH_NAME: &str = "Cedric";
const DAILY_UNIT_QUANTITY: u32 = 1;
const TRADE_PROMPT_VERB: &str = "discusses exchanging a";
const SCHEDULE_PROMPT_ACTION: &str = "reviews the day's schedule";
const SCHEDULE_SUMMARY_PREFIX: &str = "Daily plan:";
const SENTENCE_SUFFIX: &str = ".";

/// Spawns placeholder crate entities representing profession work spots.
pub fn spawn_profession_crates(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut registry: ResMut<ProfessionCrateRegistry>,
) {
    let crate_specs = [
        (
            Profession::Farmer,
            Vec3::new(8.0, 0.25, 3.0),
            Color::srgb_u8(190, 150, 80),
        ),
        (
            Profession::Miller,
            Vec3::new(0.0, 0.25, -6.5),
            Color::srgb_u8(140, 170, 215),
        ),
        (
            Profession::Blacksmith,
            Vec3::new(-6.0, 0.25, 1.5),
            Color::srgb_u8(110, 110, 130),
        ),
    ];

    for (profession, translation, color) in crate_specs {
        if registry.get(profession).is_some() {
            continue;
        }

        let entity = commands
            .spawn((
                Mesh3d(meshes.add(Mesh::from(Cuboid::new(0.9, 0.6, 0.9)))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: color,
                    perceptual_roughness: 0.6,
                    metallic: 0.1,
                    ..default()
                })),
                Transform::from_translation(translation),
                ProfessionCrate { profession },
                Name::new(format!("{} crate", profession.label())),
            ))
            .id();

        registry.insert(profession, entity);
        info!(
            "Spawned {} crate at ({:.1}, {:.1}, {:.1})",
            profession.label(),
            translation.x,
            translation.y,
            translation.z
        );
    }
}

#[derive(Clone, Copy)]
struct TradeDialogueInput {
    day: u64,
    from: Option<NpcId>,
    to: Option<NpcId>,
    good: TradeGood,
    quantity: u32,
    reason: TradeReason,
}

/// Assigns placeholder professions and empty inventories to debug NPCs.
pub fn assign_placeholder_professions(
    mut commands: Commands,
    query: Query<(Entity, &Identity), Without<Profession>>,
) {
    for (entity, identity) in query.iter() {
        let profession = match identity.display_name.as_str() {
            FARMER_NAME => Some(Profession::Farmer),
            MILLER_NAME => Some(Profession::Miller),
            BLACKSMITH_NAME => Some(Profession::Blacksmith),
            _ => None,
        };

        if let Some(profession) = profession {
            info!(
                "Assigning {} (age {:.1}) as {}",
                identity.display_name,
                identity.age_years,
                profession.label()
            );
            commands
                .entity(entity)
                .insert((profession, Inventory::default()));
        }
    }
}

/// Runs once per in-game day to simulate a simple trade loop between professions.
pub fn process_micro_trade_loop(
    world_clock: Res<WorldClock>,
    mut state: ResMut<MicroTradeLoopState>,
    identity_query: Query<(Entity, &Identity, &Profession)>,
    mut locomotion_query: Query<(&GlobalTransform, &mut NpcLocomotion)>,
    crate_transforms: Query<&GlobalTransform, With<ProfessionCrate>>,
    registry: Res<ProfessionCrateRegistry>,
    dependency_matrix: Res<EconomyDependencyMatrix>,
    mut inventories: Query<&mut Inventory>,
    mut trade_writer: MessageWriter<TradeCompletedEvent>,
    mut dependency_writer: MessageWriter<ProfessionDependencyUpdateEvent>,
    mut dialogue_queue: ResMut<DialogueRequestQueue>,
) {
    let day = world_clock.day_count();
    if state.last_processed_day == Some(day) {
        return;
    }

    let mut farmer = None;
    let mut miller = None;
    let mut blacksmith = None;

    for (entity, identity, profession) in identity_query.iter() {
        match profession {
            Profession::Farmer => {
                farmer = Some((entity, identity.id, identity.display_name.clone()))
            }
            Profession::Miller => {
                miller = Some((entity, identity.id, identity.display_name.clone()))
            }
            Profession::Blacksmith => {
                blacksmith = Some((entity, identity.id, identity.display_name.clone()))
            }
        }
    }

    let (farmer_entity, farmer_id, farmer_name) = match farmer {
        Some(data) => data,
        None => {
            warn!("Micro trade loop skipped: no farmer present");
            return;
        }
    };
    let (miller_entity, miller_id, miller_name) = match miller {
        Some(data) => data,
        None => {
            warn!("Micro trade loop skipped: no miller present");
            return;
        }
    };
    let (smith_entity, smith_id, smith_name) = match blacksmith {
        Some(data) => data,
        None => {
            warn!("Micro trade loop skipped: no blacksmith present");
            return;
        }
    };

    let workers = [
        WorkerAssignment {
            entity: farmer_entity,
            profession: Profession::Farmer,
            display_name: farmer_name.clone(),
        },
        WorkerAssignment {
            entity: miller_entity,
            profession: Profession::Miller,
            display_name: miller_name.clone(),
        },
        WorkerAssignment {
            entity: smith_entity,
            profession: Profession::Blacksmith,
            display_name: smith_name.clone(),
        },
    ];

    if !ensure_profession_workers_ready(
        day,
        &workers,
        &registry,
        &crate_transforms,
        &mut locomotion_query,
    ) {
        return;
    }

    state.last_processed_day = Some(day);

    let Ok([mut farmer_inv, mut miller_inv, mut smith_inv]) =
        inventories.get_many_mut([farmer_entity, miller_entity, smith_entity])
    else {
        warn!("Micro trade loop skipped: inventory lookup failed");
        return;
    };

    // Farmer produces grain for the day.
    farmer_inv.add_good(TradeGood::Grain, DAILY_UNIT_QUANTITY);
    trade_writer.write(TradeCompletedEvent {
        day,
        from: None,
        to: Some(farmer_id),
        good: TradeGood::Grain,
        quantity: DAILY_UNIT_QUANTITY,
        reason: TradeReason::Production,
    });
    info!("{} harvests a grain crate", farmer_name);

    // Farmer delivers grain to the miller.
    if farmer_inv.remove_good(TradeGood::Grain, DAILY_UNIT_QUANTITY) {
        miller_inv.add_good(TradeGood::Grain, DAILY_UNIT_QUANTITY);
        send_trade_and_dialogue(
            &mut trade_writer,
            &mut dialogue_queue,
            TradeDialogueInput {
                day,
                from: Some(farmer_id),
                to: Some(miller_id),
                good: TradeGood::Grain,
                quantity: DAILY_UNIT_QUANTITY,
                reason: TradeReason::Exchange,
            },
        );
        info!("{} passes grain crate to {}", farmer_name, miller_name);
    } else {
        warn!("{} has no grain crate to trade", farmer_name);
        return;
    }

    // Miller processes grain into flour.
    if miller_inv.remove_good(TradeGood::Grain, DAILY_UNIT_QUANTITY) {
        miller_inv.add_good(TradeGood::Flour, DAILY_UNIT_QUANTITY);
        trade_writer.write(TradeCompletedEvent {
            day,
            from: Some(miller_id),
            to: Some(miller_id),
            good: TradeGood::Flour,
            quantity: DAILY_UNIT_QUANTITY,
            reason: TradeReason::Processing,
        });
    } else {
        warn!("{} missing grain crate for milling", miller_name);
        return;
    }

    // Miller delivers flour to the blacksmith.
    if miller_inv.remove_good(TradeGood::Flour, DAILY_UNIT_QUANTITY) {
        smith_inv.add_good(TradeGood::Flour, DAILY_UNIT_QUANTITY);
        send_trade_and_dialogue(
            &mut trade_writer,
            &mut dialogue_queue,
            TradeDialogueInput {
                day,
                from: Some(miller_id),
                to: Some(smith_id),
                good: TradeGood::Flour,
                quantity: DAILY_UNIT_QUANTITY,
                reason: TradeReason::Exchange,
            },
        );
        info!("{} sends flour crate to {}", miller_name, smith_name);
    } else {
        warn!("{} missing flour crate for delivery", miller_name);
        return;
    }

    // Blacksmith processes flour into tool crate (placeholder transformation).
    if smith_inv.remove_good(TradeGood::Flour, DAILY_UNIT_QUANTITY) {
        smith_inv.add_good(TradeGood::Tools, DAILY_UNIT_QUANTITY);
        trade_writer.write(TradeCompletedEvent {
            day,
            from: Some(smith_id),
            to: Some(smith_id),
            good: TradeGood::Tools,
            quantity: DAILY_UNIT_QUANTITY,
            reason: TradeReason::Processing,
        });
    } else {
        warn!("{} missing flour crate to craft tools", smith_name);
        return;
    }

    // Blacksmith returns tools to the farmer.
    if smith_inv.remove_good(TradeGood::Tools, DAILY_UNIT_QUANTITY) {
        farmer_inv.add_good(TradeGood::Tools, DAILY_UNIT_QUANTITY);
        send_trade_and_dialogue(
            &mut trade_writer,
            &mut dialogue_queue,
            TradeDialogueInput {
                day,
                from: Some(smith_id),
                to: Some(farmer_id),
                good: TradeGood::Tools,
                quantity: DAILY_UNIT_QUANTITY,
                reason: TradeReason::Exchange,
            },
        );
        info!("{} supplies tool crate to {}", smith_name, farmer_name);
        queue_schedule_brief(
            &mut dialogue_queue,
            day,
            farmer_id,
            format!(
                "{} coordinated tool deliveries with {} and {}",
                farmer_name, miller_name, smith_name
            ),
        );
        debug!(
            "Inventory snapshot -> farmer: grain {} flour {} tools {}; miller: grain {} flour {}; smith: flour {} tools {}",
            farmer_inv.quantity_of(TradeGood::Grain),
            farmer_inv.quantity_of(TradeGood::Flour),
            farmer_inv.quantity_of(TradeGood::Tools),
            miller_inv.quantity_of(TradeGood::Grain),
            miller_inv.quantity_of(TradeGood::Flour),
            smith_inv.quantity_of(TradeGood::Flour),
            smith_inv.quantity_of(TradeGood::Tools),
        );
    } else {
        warn!("{} missing tool crate for delivery", smith_name);
    }

    emit_dependency_updates(
        day,
        &dependency_matrix,
        &mut dependency_writer,
        [
            (farmer_id, Profession::Farmer, &*farmer_inv),
            (miller_id, Profession::Miller, &*miller_inv),
            (smith_id, Profession::Blacksmith, &*smith_inv),
        ],
    );
}

struct WorkerAssignment {
    entity: Entity,
    profession: Profession,
    display_name: String,
}

fn emit_dependency_updates(
    day: u64,
    matrix: &EconomyDependencyMatrix,
    writer: &mut MessageWriter<ProfessionDependencyUpdateEvent>,
    snapshots: [(NpcId, Profession, &Inventory); 3],
) {
    for (npc_id, profession, inventory) in snapshots {
        let mut satisfied = Vec::new();
        let mut missing = Vec::new();
        for category in matrix.requirements(profession) {
            let goods_for_category = [TradeGood::Grain, TradeGood::Flour, TradeGood::Tools]
                .into_iter()
                .filter(|good| {
                    matrix
                        .categories_for_good(*good)
                        .iter()
                        .any(|candidate| candidate == category)
                });

            let category_met = goods_for_category.any(|good| inventory.quantity_of(good) > 0);

            if category_met {
                satisfied.push(*category);
            } else {
                missing.push(*category);
            }
        }

        writer.write(ProfessionDependencyUpdateEvent {
            day,
            npc: npc_id,
            profession,
            satisfied_categories: satisfied,
            missing_categories: missing,
        });
    }
}

fn ensure_profession_workers_ready(
    day: u64,
    workers: &[WorkerAssignment],
    registry: &ProfessionCrateRegistry,
    crate_transforms: &Query<&GlobalTransform, With<ProfessionCrate>>,
    locomotion_query: &mut Query<(&GlobalTransform, &mut NpcLocomotion)>,
) -> bool {
    let mut all_ready = true;

    for worker in workers {
        let Some(crate_entity) = registry.get(worker.profession) else {
            warn!(
                "Trade loop day {} skipped: no crate registered for {}",
                day,
                worker.profession.label()
            );
            return false;
        };

        let Ok((npc_transform, mut locomotion)) = locomotion_query.get_mut(worker.entity) else {
            warn!(
                "Trade loop day {} skipped: {} missing locomotion component",
                day, worker.display_name
            );
            return false;
        };

        let Ok(crate_transform) = crate_transforms.get(crate_entity) else {
            warn!(
                "Trade loop day {} skipped: crate entity {:?} missing transform",
                day, crate_entity
            );
            return false;
        };

        let current: Vec3 = npc_transform.translation().into();
        let mut target: Vec3 = crate_transform.translation().into();
        target.y = current.y;

        let displacement = Vec2::new(target.x - current.x, target.z - current.z);
        let distance = displacement.length();
        let tolerance = locomotion.arrive_distance();

        if distance > tolerance {
            let label = format!("{} crate", worker.profession.label());
            if locomotion.set_target(MovementTarget::Entity(crate_entity), label.clone()) {
                info!("{} starts walking toward {}", worker.display_name, label);
            }
            all_ready = false;
        }
    }

    if !all_ready {
        debug!(
            "Day {} trade loop paused: workers still travelling to crates",
            day
        );
    }

    all_ready
}

fn queue_schedule_brief(
    queue: &mut DialogueRequestQueue,
    day: u64,
    speaker: NpcId,
    description: String,
) {
    let mut context =
        DialogueContext::with_events(vec![DialogueContextEvent::ScheduleUpdate { description }]);
    context.summary = Some(format!("{SCHEDULE_SUMMARY_PREFIX} Day {day}"));

    let prompt = format!(
        "{speaker} {action}{suffix}",
        speaker = speaker,
        action = SCHEDULE_PROMPT_ACTION,
        suffix = SENTENCE_SUFFIX
    );

    let request = DialogueRequest::new(speaker, None, prompt, DialogueTopicHint::Schedule, context);
    let id = queue.enqueue(request);
    debug!(
        "Queued schedule update dialogue {} for speaker {} on day {}",
        id.value(),
        speaker,
        day
    );
}

fn send_trade_and_dialogue(
    trade_writer: &mut MessageWriter<TradeCompletedEvent>,
    queue: &mut DialogueRequestQueue,
    input: TradeDialogueInput,
) {
    trade_writer.write(TradeCompletedEvent {
        day: input.day,
        from: input.from,
        to: input.to,
        good: input.good,
        quantity: input.quantity,
        reason: input.reason,
    });

    if let (Some(speaker), Some(target)) = (input.from, input.to) {
        let descriptor = TradeDescriptor::new(input.good.label(), input.quantity);
        let context =
            DialogueContext::with_events(vec![DialogueContextEvent::Trade(TradeContext {
                day: input.day,
                from: input.from,
                to: input.to,
                descriptor,
                reason: input.reason.into(),
            })]);
        let prompt = build_trade_prompt(speaker, input.good.label());
        let request = DialogueRequest::new(
            speaker,
            Some(target),
            prompt,
            DialogueTopicHint::Trade,
            context,
        );
        let id = queue.enqueue(request);
        debug!("Queued dialogue request {} for trade", id.value());
    }
}

impl From<TradeReason> for TradeContextReason {
    fn from(value: TradeReason) -> Self {
        match value {
            TradeReason::Production => TradeContextReason::Production,
            TradeReason::Processing => TradeContextReason::Processing,
            TradeReason::Exchange => TradeContextReason::Exchange,
        }
    }
}

fn build_trade_prompt(speaker: NpcId, good_label: &str) -> String {
    format!(
        "{speaker} {verb} {good}{suffix}",
        speaker = speaker,
        verb = TRADE_PROMPT_VERB,
        good = good_label,
        suffix = SENTENCE_SUFFIX
    )
}
