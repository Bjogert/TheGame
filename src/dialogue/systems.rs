//! Systems driving the dialogue queue lifecycle.
use bevy::prelude::*;

use crate::core::plugin::SimulationClock;

use super::{
    broker::{DialogueBrokerRegistry, DialogueError, DialogueSubmissionStatus},
    queue::DialogueRequestQueue,
};

const MAX_DISPATCH_PER_FRAME: usize = 16;

pub fn run_dialogue_queue(
    mut queue: ResMut<DialogueRequestQueue>,
    registry: Res<DialogueBrokerRegistry>,
    clock: Res<SimulationClock>,
) {
    let delta_seconds = clock.last_scaled_delta().as_secs_f32();
    queue.tick(delta_seconds);

    let limit = queue
        .config()
        .max_dispatches_per_tick
        .min(MAX_DISPATCH_PER_FRAME);
    let ready = queue.take_ready_limit(limit);

    if ready.is_empty() {
        return;
    }

    for request in ready {
        match registry.dispatch(&request) {
            Ok(submission) => match submission.status {
                DialogueSubmissionStatus::Accepted { estimated_latency } => {
                    queue.record_accept(&request, estimated_latency);
                }
                DialogueSubmissionStatus::Deferred { retry_after } => {
                    queue.record_deferred(request, retry_after);
                }
            },
            Err(error) => queue.record_error(request, error),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::VecDeque, sync::Mutex, time::Duration};

    use super::*;
    use crate::dialogue::broker::{DialogueProvider, DialogueRequestId, DialogueSubmission};
    use crate::dialogue::queue::DialogueRequest;

    struct TestBroker {
        responses: Mutex<VecDeque<Result<DialogueSubmission, DialogueError>>>,
    }

    impl TestBroker {
        fn new(responses: Vec<Result<DialogueSubmission, DialogueError>>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
            }
        }
    }

    impl super::super::broker::DialogueBroker for TestBroker {
        fn provider(&self) -> DialogueProvider {
            DialogueProvider::Local
        }

        fn submit(&self, _request: &DialogueRequest) -> Result<DialogueSubmission, DialogueError> {
            self.responses
                .lock()
                .expect("mutex poisoned")
                .pop_front()
                .unwrap_or_else(|| {
                    Err(DialogueError::Cancelled {
                        request_id: DialogueRequestId::new(0),
                        reason: "no response".into(),
                    })
                })
        }
    }

    #[test]
    fn processes_ready_requests() {
        let mut app = App::new();
        app.add_systems(Update, run_dialogue_queue);

        let mut queue = DialogueRequestQueue::default();
        let request_id = queue.enqueue(DialogueProvider::Local, None, "hello");

        let submission = DialogueSubmission {
            request_id,
            status: DialogueSubmissionStatus::Accepted {
                estimated_latency: Duration::from_millis(10),
            },
        };

        let broker = TestBroker::new(vec![Ok(submission)]);

        let mut registry = DialogueBrokerRegistry::default();
        registry.register(Box::new(broker));

        app.insert_resource(queue);
        app.insert_resource(registry);
        app.insert_resource(SimulationClock::default());

        app.update();

        let queue = app.world().resource::<DialogueRequestQueue>();
        assert_eq!(queue.metrics().accepted, 1);
    }
}
