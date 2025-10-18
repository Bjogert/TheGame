use bevy::prelude::*;

use crate::world::time::WorldClock;

use super::super::{
    data::EconomyRegistry,
    planning::schedule_daily_requests,
    tasks::{ActorTaskQueues, EconomyDayState},
};

/// Prepares the list of tasks each economy actor should complete for the current day.
pub fn prepare_economy_day(
    world_clock: Res<WorldClock>,
    registry: Res<EconomyRegistry>,
    mut day_state: ResMut<EconomyDayState>,
    mut task_queues: ResMut<ActorTaskQueues>,
) {
    let day = world_clock.day_count();
    if day_state.last_planned_day == Some(day) {
        return;
    }

    task_queues.clear();

    if let Err(error) = schedule_daily_requests(&registry, &mut task_queues) {
        warn!("Unable to schedule economy tasks for day {day}: {error}");
        return;
    }

    day_state.last_planned_day = Some(day);
    day_state.last_dependency_evaluation_day = None;

    for profession in task_queues.professions() {
        debug!(
            "Planned {} tasks for {}",
            task_queues.remaining_tasks(profession),
            profession.label()
        );
    }
}
