//! Planner that converts economy requests into actor task queues.
use std::collections::HashMap;

use super::{
    components::{Profession, TradeGood},
    data::{DailyRequest, EconomyRegistry},
    tasks::{ActorTask, ActorTaskQueues},
};

pub fn schedule_daily_requests(
    registry: &EconomyRegistry,
    queues: &mut ActorTaskQueues,
) -> Result<(), String> {
    for request in registry.daily_requests() {
        schedule_request(registry, queues, request)?;
    }
    Ok(())
}

fn schedule_request(
    registry: &EconomyRegistry,
    queues: &mut ActorTaskQueues,
    request: &DailyRequest,
) -> Result<(), String> {
    for _ in 0..request.quantity {
        let mut pending: HashMap<Profession, Vec<ActorTask>> = HashMap::new();
        let producer = plan_request_unit(registry, request.good, request.requester, &mut pending)?;

        if producer != request.requester {
            pending
                .entry(request.requester)
                .or_default()
                .push(ActorTask::WaitForGood {
                    good: request.good,
                    quantity: 1,
                });
        }

        for (profession, tasks) in pending {
            queues.ensure_queue(profession).extend(tasks.into_iter());
        }
    }

    Ok(())
}

fn plan_request_unit(
    registry: &EconomyRegistry,
    good: TradeGood,
    target: Profession,
    tasks: &mut HashMap<Profession, Vec<ActorTask>>,
) -> Result<Profession, String> {
    let recipe = registry
        .recipe_for_output(good)
        .ok_or_else(|| format!("no recipe produces good {:?}", good))?;

    let mut total_outputs = 0;
    for output in &recipe.produces {
        if output.good == good {
            total_outputs += output.quantity.max(1);
        }
    }

    if total_outputs == 0 {
        return Err(format!(
            "recipe '{}' does not produce requested good {:?}",
            recipe.id, good
        ));
    }

    for input in &recipe.consumes {
        for _ in 0..input.quantity.max(1) {
            let _supplier = plan_request_unit(registry, input.good, recipe.actor, tasks)?;
            tasks
                .entry(recipe.actor)
                .or_default()
                .push(ActorTask::WaitForGood {
                    good: input.good,
                    quantity: 1,
                });
        }
    }

    tasks
        .entry(recipe.actor)
        .or_default()
        .push(ActorTask::Manufacture {
            recipe_id: recipe.id.clone(),
        });

    for _ in 0..total_outputs {
        tasks
            .entry(recipe.actor)
            .or_default()
            .push(ActorTask::Deliver {
                good,
                quantity: 1,
                target,
            });
    }

    Ok(recipe.actor)
}
