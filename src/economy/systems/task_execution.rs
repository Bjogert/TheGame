use std::collections::HashMap;

use bevy::{
    ecs::system::{ParamSet, SystemParam},
    prelude::*,
};

use crate::{
    dialogue::queue::DialogueRequestQueue,
    npc::components::{Identity, LocomotionState, MovementTarget, NpcId, NpcLocomotion},
    world::time::WorldClock,
};

use super::{
    super::{
        components::{Inventory, Profession, ProfessionCrate, TradeGood, TradeGoodPlaceholder},
        data::EconomyRegistry,
        dependency::EconomyDependencyMatrix,
        events::{ProfessionDependencyUpdateEvent, TradeCompletedEvent, TradeReason},
        resources::{
            ProfessionCrateRegistry, TradeGoodPlaceholderRegistry, TradeGoodPlaceholderVisuals,
        },
        tasks::{ActorTask, ActorTaskQueues, EconomyDayState},
    },
    dialogue::{queue_schedule_brief, send_trade_and_dialogue, TradeDialogueInput},
    spawning::{BLACKSMITH_NAME, FARMER_NAME, MILLER_NAME},
};

const ALL_TRADE_GOODS: [TradeGood; 3] = [TradeGood::Grain, TradeGood::Flour, TradeGood::Tools];
const GRAIN_PLACEHOLDER_OFFSET: Vec3 = Vec3::new(0.35, 0.55, 0.0);
const FLOUR_PLACEHOLDER_OFFSET: Vec3 = Vec3::new(-0.35, 0.55, 0.0);
const TOOLS_PLACEHOLDER_OFFSET: Vec3 = Vec3::new(0.0, 0.6, 0.35);

/// Runs the queued tasks for each profession, driving production and trade.
#[allow(clippy::too_many_arguments)]
pub fn advance_actor_tasks(
    mut commands: Commands,
    world_clock: Res<WorldClock>,
    registry: Res<EconomyRegistry>,
    dependency_matrix: Res<EconomyDependencyMatrix>,
    mut day_state: ResMut<EconomyDayState>,
    mut task_queues: ResMut<ActorTaskQueues>,
    mut placeholders: ResMut<TradeGoodPlaceholderRegistry>,
    crate_registry: Res<ProfessionCrateRegistry>,
    mut inventory_queries: ParamSet<(Query<&mut Inventory>, Query<&Inventory>)>,
    mut locomotion_query: Query<(&GlobalTransform, &mut NpcLocomotion)>,
    crate_transforms: Query<&GlobalTransform, With<ProfessionCrate>>,
    identity_query: Query<(Entity, &Identity, &Profession)>,
    mut outputs: EconomyOutputs,
    visuals: Res<TradeGoodPlaceholderVisuals>,
) {
    if task_queues.is_empty() {
        if let Some(day) = day_state.last_planned_day {
            if day_state.last_dependency_evaluation_day != Some(day) {
                {
                    let inventory_ro = inventory_queries.p1();
                    emit_dependency_updates(
                        day,
                        &dependency_matrix,
                        &mut outputs.dependency_writer,
                        &identity_query,
                        &inventory_ro,
                    );
                }
                day_state.last_dependency_evaluation_day = Some(day);
            }
        }
        return;
    }

    let actor_map = match collect_actor_data(&identity_query) {
        Some(map) => map,
        None => {
            debug!("Economy tasks paused: missing profession assignments");
            return;
        }
    };

    let professions: Vec<Profession> = task_queues.professions().collect();
    let mut all_complete = true;

    for profession in professions {
        let Some(task) = task_queues.peek_mut(profession) else {
            continue;
        };

        let Some(actor) = actor_map.get(&profession) else {
            warn!(
                "Skipping tasks for {}: profession not assigned to any NPC",
                profession.label()
            );
            task_queues.pop_front(profession);
            continue;
        };

        match execute_task(
            &mut commands,
            &registry,
            &crate_registry,
            &crate_transforms,
            &actor_map,
            profession,
            actor,
            task,
            world_clock.day_count(),
            &mut locomotion_query,
            &mut inventory_queries,
            &mut placeholders,
            &mut outputs,
            visuals.as_ref(),
        ) {
            TaskResult::Completed => {
                task_queues.pop_front(profession);
            }
            TaskResult::InProgress => {
                all_complete = false;
            }
        }
    }

    if all_complete && task_queues.is_empty() {
        if let Some(day) = day_state.last_planned_day {
            let inventory_ro = inventory_queries.p1();
            emit_dependency_updates(
                day,
                &dependency_matrix,
                &mut outputs.dependency_writer,
                &identity_query,
                &inventory_ro,
            );
            day_state.last_dependency_evaluation_day = Some(day);
        }
    }
}

#[derive(SystemParam)]
pub struct EconomyOutputs<'w> {
    trade_writer: MessageWriter<'w, TradeCompletedEvent>,
    dependency_writer: MessageWriter<'w, ProfessionDependencyUpdateEvent>,
    dialogue_queue: ResMut<'w, DialogueRequestQueue>,
}

#[derive(Debug)]
struct ActorData {
    entity: Entity,
    npc_id: NpcId,
    display_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskResult {
    Completed,
    InProgress,
}

fn collect_actor_data(
    query: &Query<(Entity, &Identity, &Profession)>,
) -> Option<HashMap<Profession, ActorData>> {
    let mut actors = HashMap::new();
    for (entity, identity, profession) in query.iter() {
        actors.insert(
            *profession,
            ActorData {
                entity,
                npc_id: identity.id,
                display_name: identity.display_name.clone(),
            },
        );
    }

    if actors.len() < 3 {
        return None;
    }

    Some(actors)
}

#[allow(clippy::too_many_arguments)]
fn execute_task(
    commands: &mut Commands,
    registry: &EconomyRegistry,
    crate_registry: &ProfessionCrateRegistry,
    crate_transforms: &Query<&GlobalTransform, With<ProfessionCrate>>,
    actor_map: &HashMap<Profession, ActorData>,
    profession: Profession,
    actor: &ActorData,
    task: &mut ActorTask,
    day: u64,
    locomotion_query: &mut Query<(&GlobalTransform, &mut NpcLocomotion)>,
    inventory_queries: &mut ParamSet<(Query<&mut Inventory>, Query<&Inventory>)>,
    placeholders: &mut TradeGoodPlaceholderRegistry,
    outputs: &mut EconomyOutputs,
    visuals: &TradeGoodPlaceholderVisuals,
) -> TaskResult {
    match task.clone() {
        ActorTask::WaitForGood { good, quantity } => execute_wait_for_good(
            crate_registry,
            crate_transforms,
            profession,
            actor,
            good,
            quantity,
            locomotion_query,
            inventory_queries,
        ),
        ActorTask::Manufacture { recipe_id } => execute_manufacture(
            commands,
            registry,
            crate_registry,
            crate_transforms,
            visuals,
            profession,
            actor,
            &recipe_id,
            day,
            locomotion_query,
            inventory_queries,
            placeholders,
            &mut outputs.trade_writer,
        ),
        ActorTask::Deliver {
            good,
            quantity,
            target,
        } => execute_deliver(
            commands,
            crate_registry,
            crate_transforms,
            actor_map,
            visuals,
            profession,
            actor,
            target,
            good,
            quantity,
            day,
            locomotion_query,
            inventory_queries,
            placeholders,
            &mut outputs.trade_writer,
            outputs.dialogue_queue.as_mut(),
        ),
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_wait_for_good(
    crate_registry: &ProfessionCrateRegistry,
    crate_transforms: &Query<&GlobalTransform, With<ProfessionCrate>>,
    profession: Profession,
    actor: &ActorData,
    good: TradeGood,
    quantity: u32,
    locomotion_query: &mut Query<(&GlobalTransform, &mut NpcLocomotion)>,
    inventory_queries: &mut ParamSet<(Query<&mut Inventory>, Query<&Inventory>)>,
) -> TaskResult {
    if !ensure_actor_at_location(
        profession,
        profession,
        actor,
        crate_registry,
        crate_transforms,
        locomotion_query,
    ) {
        return TaskResult::InProgress;
    }

    let inventories = inventory_queries.p1();
    if let Ok(inventory) = inventories.get(actor.entity) {
        if inventory.quantity_of(good) >= quantity {
            TaskResult::Completed
        } else {
            TaskResult::InProgress
        }
    } else {
        warn!(
            "{} is missing an inventory; cannot wait for goods",
            actor.display_name
        );
        TaskResult::Completed
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_manufacture(
    commands: &mut Commands,
    registry: &EconomyRegistry,
    crate_registry: &ProfessionCrateRegistry,
    crate_transforms: &Query<&GlobalTransform, With<ProfessionCrate>>,
    visuals: &TradeGoodPlaceholderVisuals,
    profession: Profession,
    actor: &ActorData,
    recipe_id: &str,
    day: u64,
    locomotion_query: &mut Query<(&GlobalTransform, &mut NpcLocomotion)>,
    inventory_queries: &mut ParamSet<(Query<&mut Inventory>, Query<&Inventory>)>,
    placeholders: &mut TradeGoodPlaceholderRegistry,
    trade_writer: &mut MessageWriter<TradeCompletedEvent>,
) -> TaskResult {
    if !ensure_actor_at_location(
        profession,
        profession,
        actor,
        crate_registry,
        crate_transforms,
        locomotion_query,
    ) {
        return TaskResult::InProgress;
    }

    let Some(recipe) = registry.recipe(recipe_id) else {
        warn!(
            "{} cannot manufacture: recipe '{}' missing",
            actor.display_name, recipe_id
        );
        return TaskResult::Completed;
    };

    {
        let inventories = inventory_queries.p1();
        if let Ok(inventory) = inventories.get(actor.entity) {
            for input in &recipe.consumes {
                if inventory.quantity_of(input.good) < input.quantity {
                    return TaskResult::InProgress;
                }
            }
        } else {
            warn!(
                "{} is missing an inventory; cannot manufacture goods",
                actor.display_name
            );
            return TaskResult::Completed;
        }
    }

    let mut inventories = inventory_queries.p0();
    let Ok(mut inventory) = inventories.get_mut(actor.entity) else {
        warn!(
            "{} is missing an inventory; cannot manufacture goods",
            actor.display_name
        );
        return TaskResult::Completed;
    };

    for input in &recipe.consumes {
        if inventory.remove_good(input.good, input.quantity)
            && inventory.quantity_of(input.good) == 0
        {
            despawn_trade_good_placeholder(commands, placeholders, profession, input.good);
        }
    }

    for output in &recipe.produces {
        let previous = inventory.quantity_of(output.good);
        inventory.add_good(output.good, output.quantity);
        if previous == 0 {
            spawn_trade_good_placeholder(
                commands,
                placeholders,
                crate_registry,
                visuals,
                profession,
                output.good,
            );
        }

        let reason = if recipe.consumes.is_empty() {
            TradeReason::Production
        } else {
            TradeReason::Processing
        };

        trade_writer.write(TradeCompletedEvent {
            day,
            from: Some(actor.npc_id),
            to: Some(actor.npc_id),
            good: output.good,
            quantity: output.quantity,
            reason,
        });
    }

    TaskResult::Completed
}

#[allow(clippy::too_many_arguments)]
fn execute_deliver(
    commands: &mut Commands,
    crate_registry: &ProfessionCrateRegistry,
    crate_transforms: &Query<&GlobalTransform, With<ProfessionCrate>>,
    actor_map: &HashMap<Profession, ActorData>,
    visuals: &TradeGoodPlaceholderVisuals,
    profession: Profession,
    actor: &ActorData,
    target: Profession,
    good: TradeGood,
    quantity: u32,
    day: u64,
    locomotion_query: &mut Query<(&GlobalTransform, &mut NpcLocomotion)>,
    inventory_queries: &mut ParamSet<(Query<&mut Inventory>, Query<&Inventory>)>,
    placeholders: &mut TradeGoodPlaceholderRegistry,
    trade_writer: &mut MessageWriter<TradeCompletedEvent>,
    dialogue_queue: &mut DialogueRequestQueue,
) -> TaskResult {
    if !ensure_actor_at_location(
        profession,
        target,
        actor,
        crate_registry,
        crate_transforms,
        locomotion_query,
    ) {
        return TaskResult::InProgress;
    }

    let Some(target_actor) = actor_map.get(&target) else {
        warn!(
            "{} attempted delivery to missing {}",
            actor.display_name,
            target.label()
        );
        return TaskResult::Completed;
    };

    {
        let mut inventories = inventory_queries.p0();
        let Ok(mut inventory) = inventories.get_mut(actor.entity) else {
            warn!(
                "{} is missing an inventory; delivery cancelled",
                actor.display_name
            );
            return TaskResult::Completed;
        };

        if inventory.quantity_of(good) < quantity {
            return TaskResult::InProgress;
        }

        if !inventory.remove_good(good, quantity) {
            return TaskResult::InProgress;
        }

        if inventory.quantity_of(good) == 0 {
            despawn_trade_good_placeholder(commands, placeholders, profession, good);
        }
    }

    {
        let mut inventories = inventory_queries.p0();
        if let Ok(mut target_inventory) = inventories.get_mut(target_actor.entity) {
            let previous = target_inventory.quantity_of(good);
            target_inventory.add_good(good, quantity);
            if previous == 0 {
                spawn_trade_good_placeholder(
                    commands,
                    placeholders,
                    crate_registry,
                    visuals,
                    target,
                    good,
                );
            }
        } else {
            warn!(
                "{} is missing an inventory; delivery from {} discarded",
                target_actor.display_name, actor.display_name
            );
        }
    }

    send_trade_and_dialogue(
        trade_writer,
        dialogue_queue,
        TradeDialogueInput {
            day,
            from: Some(actor.npc_id),
            to: Some(target_actor.npc_id),
            good,
            quantity,
            reason: TradeReason::Exchange,
        },
    );

    if target == Profession::Farmer && good == TradeGood::Tools {
        queue_schedule_brief(
            dialogue_queue,
            day,
            target_actor.npc_id,
            format!(
                "{} coordinated trades with {} and {}",
                target_actor.display_name, MILLER_NAME, BLACKSMITH_NAME
            ),
        );
    }

    TaskResult::Completed
}

#[allow(clippy::too_many_arguments)]
fn ensure_actor_at_location(
    movement_owner: Profession,
    location_owner: Profession,
    actor: &ActorData,
    crate_registry: &ProfessionCrateRegistry,
    crate_transforms: &Query<&GlobalTransform, With<ProfessionCrate>>,
    locomotion_query: &mut Query<(&GlobalTransform, &mut NpcLocomotion)>,
) -> bool {
    let Some(crate_entity) = crate_registry.get(location_owner) else {
        warn!("No crate registered for {}", location_owner.label());
        return true;
    };

    let Ok((actor_transform, mut locomotion)) = locomotion_query.get_mut(actor.entity) else {
        warn!("{} is missing locomotion data", actor.display_name);
        return true;
    };

    let Ok(crate_transform) = crate_transforms.get(crate_entity) else {
        warn!(
            "Crate entity for {} missing transform",
            location_owner.label()
        );
        return true;
    };

    let current: Vec3 = actor_transform.translation().into();
    let mut target: Vec3 = crate_transform.translation().into();
    target.y = current.y;

    let displacement = Vec2::new(target.x - current.x, target.z - current.z);
    if displacement.length() <= locomotion.arrive_distance() {
        if locomotion.state() == LocomotionState::Moving {
            locomotion.clear_target();
        }
        return true;
    }

    let label = if movement_owner == location_owner {
        format!("{} crate", movement_owner.label())
    } else {
        format!("{} crate (visiting)", location_owner.label())
    };

    if locomotion.set_target(MovementTarget::Entity(crate_entity), label.clone()) {
        info!("{} starts walking toward {}", actor.display_name, label);
    }

    false
}

#[allow(clippy::too_many_arguments)]
fn emit_dependency_updates(
    day: u64,
    matrix: &EconomyDependencyMatrix,
    writer: &mut MessageWriter<ProfessionDependencyUpdateEvent>,
    identity_query: &Query<(Entity, &Identity, &Profession)>,
    inventories: &Query<&Inventory>,
) {
    for (entity, identity, profession) in identity_query.iter() {
        let Ok(inventory) = inventories.get(entity) else {
            warn!(
                "{} missing inventory; skipping dependency update",
                identity.display_name
            );
            continue;
        };

        let mut satisfied = Vec::new();
        let mut missing = Vec::new();
        for category in matrix.requirements(*profession) {
            let category_met = ALL_TRADE_GOODS.iter().any(|good| {
                matrix
                    .categories_for_good(*good)
                    .iter()
                    .any(|candidate| candidate == category)
                    && inventory.quantity_of(*good) > 0
            });

            if category_met {
                satisfied.push(*category);
            } else {
                missing.push(*category);
            }
        }

        writer.write(ProfessionDependencyUpdateEvent {
            day,
            npc: identity.id,
            profession: *profession,
            satisfied_categories: satisfied,
            missing_categories: missing,
        });
    }
}

fn spawn_trade_good_placeholder(
    commands: &mut Commands,
    placeholders: &mut TradeGoodPlaceholderRegistry,
    crate_registry: &ProfessionCrateRegistry,
    visuals: &TradeGoodPlaceholderVisuals,
    profession: Profession,
    good: TradeGood,
) {
    if placeholders.contains(profession, good) {
        return;
    }

    let Some(crate_entity) = crate_registry.get(profession) else {
        warn!(
            "Skipping placeholder spawn: no crate registered for {}",
            profession.label()
        );
        return;
    };

    let entity = commands
        .spawn((
            Mesh3d(visuals.mesh()),
            MeshMaterial3d(visuals.material(good)),
            Transform::from_translation(trade_good_offset(good)),
            TradeGoodPlaceholder { profession, good },
            Name::new(format!("{} {}", profession.label(), good.label())),
        ))
        .id();

    commands.entity(crate_entity).add_child(entity);
    placeholders.insert(profession, good, entity);
}

fn despawn_trade_good_placeholder(
    commands: &mut Commands,
    placeholders: &mut TradeGoodPlaceholderRegistry,
    profession: Profession,
    good: TradeGood,
) {
    if let Some(entity) = placeholders.take(profession, good) {
        commands.entity(entity).despawn();
    }
}

fn trade_good_offset(good: TradeGood) -> Vec3 {
    match good {
        TradeGood::Grain => GRAIN_PLACEHOLDER_OFFSET,
        TradeGood::Flour => FLOUR_PLACEHOLDER_OFFSET,
        TradeGood::Tools => TOOLS_PLACEHOLDER_OFFSET,
    }
}
