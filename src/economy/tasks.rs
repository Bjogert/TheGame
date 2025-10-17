//! Work order task queues for economy actors.
use std::collections::{HashMap, VecDeque};

use bevy::prelude::Resource;

use super::components::{Profession, TradeGood};

#[derive(Debug, Clone)]
pub enum ActorTask {
    WaitForGood {
        good: TradeGood,
        quantity: u32,
    },
    Manufacture {
        recipe_id: String,
    },
    Deliver {
        good: TradeGood,
        quantity: u32,
        target: Profession,
    },
}

#[derive(Resource, Debug, Default)]
pub struct ActorTaskQueues {
    queues: HashMap<Profession, VecDeque<ActorTask>>,
}

impl ActorTaskQueues {
    pub fn clear(&mut self) {
        self.queues.clear();
    }

    pub fn peek_mut(&mut self, profession: Profession) -> Option<&mut ActorTask> {
        self.queues
            .get_mut(&profession)
            .and_then(VecDeque::front_mut)
    }

    pub fn pop_front(&mut self, profession: Profession) {
        if let Some(queue) = self.queues.get_mut(&profession) {
            queue.pop_front();
            if queue.is_empty() {
                self.queues.remove(&profession);
            }
        }
    }

    pub fn remaining_tasks(&self, profession: Profession) -> usize {
        self.queues.get(&profession).map(|q| q.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.queues.is_empty()
    }

    pub fn professions(&self) -> impl Iterator<Item = Profession> + '_ {
        self.queues.keys().copied()
    }

    pub fn ensure_queue(&mut self, profession: Profession) -> &mut VecDeque<ActorTask> {
        self.queues.entry(profession).or_default()
    }
}

#[derive(Resource, Debug, Default)]
pub struct EconomyDayState {
    pub last_planned_day: Option<u64>,
    pub last_dependency_evaluation_day: Option<u64>,
}
