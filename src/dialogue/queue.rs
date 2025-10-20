//! Dialogue request queue and rate limiting resources.
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

use bevy::{
    prelude::*,
    tasks::{block_on, poll_once, AsyncComputeTaskPool, Task},
};

use crate::npc::components::NpcId;

use super::{
    broker::DialogueBroker,
    errors::{DialogueError, DialogueErrorKind},
    events::{DialogueRequestFailedEvent, DialogueResponseEvent},
    types::{DialogueRequest, DialogueRequestId},
};

const DEFAULT_GLOBAL_COOLDOWN_SECONDS: f32 = 1.5;
const DEFAULT_PER_NPC_COOLDOWN_SECONDS: f32 = 8.0;
const DEFAULT_MAX_RETRIES: u8 = 2;
const DEFAULT_RETRY_BACKOFF_SECONDS: f32 = 5.0;

/// Configurable rate limit values for the dialogue queue.
#[derive(Resource, Debug, Clone)]
pub struct DialogueRateLimitConfig {
    pub global_cooldown_seconds: f32,
    pub per_npc_cooldown_seconds: f32,
    pub max_retries: u8,
    pub retry_backoff_seconds: f32,
}

impl Default for DialogueRateLimitConfig {
    fn default() -> Self {
        Self {
            global_cooldown_seconds: DEFAULT_GLOBAL_COOLDOWN_SECONDS,
            per_npc_cooldown_seconds: DEFAULT_PER_NPC_COOLDOWN_SECONDS,
            max_retries: DEFAULT_MAX_RETRIES,
            retry_backoff_seconds: DEFAULT_RETRY_BACKOFF_SECONDS,
        }
    }
}

/// Tracks the remaining time until requests can be processed again.
#[derive(Resource, Debug, Default)]
pub struct DialogueRateLimitState {
    pub global_remaining: f32,
    pub npc_remaining: HashMap<NpcId, f32>,
}

impl DialogueRateLimitState {
    pub fn tick(&mut self, delta_seconds: f32) {
        let delta = delta_seconds.max(0.0);
        if self.global_remaining > 0.0 {
            self.global_remaining = (self.global_remaining - delta).max(0.0);
        }

        for cooldown in self.npc_remaining.values_mut() {
            if *cooldown > 0.0 {
                *cooldown = (*cooldown - delta).max(0.0);
            }
        }
    }

    pub fn can_process(&self, speaker: NpcId) -> bool {
        if self.global_remaining > 0.0 {
            return false;
        }
        !matches!(self.npc_remaining.get(&speaker), Some(value) if *value > 0.0)
    }

    pub fn record_success(&mut self, speaker: NpcId, config: &DialogueRateLimitConfig) {
        self.global_remaining = config.global_cooldown_seconds.max(0.0);
        self.npc_remaining
            .insert(speaker, config.per_npc_cooldown_seconds.max(0.0));
    }

    pub fn apply_backoff(&mut self, speaker: NpcId, seconds: f32) {
        let backoff = seconds.max(0.0);
        self.global_remaining = self.global_remaining.max(backoff);
        self.npc_remaining
            .entry(speaker)
            .and_modify(|value| *value = value.max(backoff))
            .or_insert(backoff);
    }
}

/// Result of a dialogue task (success or failure with retry info).
type DialogueTaskResult = (
    DialogueRequestId,
    DialogueRequest, // Original request for retry
    Result<super::types::DialogueResponse, DialogueError>,
    u8, // attempts
);

/// Resource tracking background dialogue processing tasks.
///
/// These tasks run blocking HTTP requests to OpenAI in a background thread pool
/// to prevent freezing the main game thread.
#[derive(Resource, Default)]
pub struct PendingDialogueTasks {
    tasks: Vec<Task<DialogueTaskResult>>,
}

/// Resource holding pending dialogue requests.
#[derive(Resource, Default)]
pub struct DialogueRequestQueue {
    next_request_id: u64,
    pending: VecDeque<QueuedDialogueRequest>,
}

impl DialogueRequestQueue {
    pub fn enqueue(&mut self, request: DialogueRequest) -> DialogueRequestId {
        let id = DialogueRequestId::new(self.next_request_id);
        self.next_request_id = self.next_request_id.wrapping_add(1);
        self.pending.push_back(QueuedDialogueRequest {
            id,
            request,
            attempts: 0,
            cooldown_remaining: 0.0,
        });
        id
    }

    pub fn enqueue_with_cooldown(
        &mut self,
        request: DialogueRequest,
        cooldown_seconds: f32,
    ) -> DialogueRequestId {
        let id = DialogueRequestId::new(self.next_request_id);
        self.next_request_id = self.next_request_id.wrapping_add(1);
        self.pending.push_back(QueuedDialogueRequest {
            id,
            request,
            attempts: 0,
            cooldown_remaining: cooldown_seconds.max(0.0),
        });
        id
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn front_ready(&self) -> bool {
        self.pending
            .front()
            .map(|req| req.cooldown_remaining <= 0.0)
            .unwrap_or(false)
    }

    fn tick(&mut self, delta_seconds: f32) {
        let delta = delta_seconds.max(0.0);
        for req in &mut self.pending {
            if req.cooldown_remaining > 0.0 {
                req.cooldown_remaining = (req.cooldown_remaining - delta).max(0.0);
            }
        }
    }
}

/// Wrapper for a dynamic dialogue broker instance.
///
/// Uses Arc internally to allow cheap cloning for background tasks.
#[derive(Resource, Clone)]
pub struct ActiveDialogueBroker {
    inner: Arc<Box<dyn DialogueBroker>>,
}

impl ActiveDialogueBroker {
    pub fn new(inner: Box<dyn DialogueBroker>) -> Self {
        Self {
            inner: Arc::new(inner),
        }
    }

    pub fn process(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> Result<super::types::DialogueResponse, DialogueError> {
        self.inner.process(request_id, request)
    }
}

/// Internal queue entry storing retry metadata.
#[derive(Debug, Clone)]
struct QueuedDialogueRequest {
    id: DialogueRequestId,
    request: DialogueRequest,
    attempts: u8,
    cooldown_remaining: f32,
}

/// Advances rate-limiter and per-request cooldown timers.
pub fn advance_dialogue_queue_timers(
    time: Res<Time>,
    mut queue: ResMut<DialogueRequestQueue>,
    mut limits: ResMut<DialogueRateLimitState>,
) {
    let delta = time.delta_secs().max(0.0);
    queue.tick(delta);
    limits.tick(delta);
}

/// Spawns dialogue requests to background tasks if rate limits allow.
///
/// This prevents blocking the main thread during HTTP requests to OpenAI.
pub fn run_dialogue_request_queue(
    mut queue: ResMut<DialogueRequestQueue>,
    limits: Res<DialogueRateLimitState>,
    broker: Res<ActiveDialogueBroker>,
    mut pending_tasks: ResMut<PendingDialogueTasks>,
) {
    if queue.is_empty() {
        return;
    }

    if !queue.front_ready() {
        return;
    }

    let Some(queued) = queue.pending.pop_front() else {
        return;
    };

    if !limits.can_process(queued.request.speaker) {
        queue.pending.push_front(queued);
        return;
    }

    // Clone data needed for the background task
    let request_id = queued.id;
    let request = queued.request.clone();
    let attempts = queued.attempts;
    let broker_clone = broker.clone();

    // Spawn to background thread to avoid blocking the game
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
        let result = broker_clone.process(request_id, &request);
        (request_id, request.clone(), result, attempts)
    });

    pending_tasks.tasks.push(task);
}

/// Polls completed dialogue tasks and emits events.
///
/// Runs every frame to check if any background dialogue requests have finished.
pub fn poll_dialogue_tasks(
    mut pending_tasks: ResMut<PendingDialogueTasks>,
    mut queue: ResMut<DialogueRequestQueue>,
    mut limits: ResMut<DialogueRateLimitState>,
    config: Res<DialogueRateLimitConfig>,
    mut response_writer: MessageWriter<DialogueResponseEvent>,
    mut failure_writer: MessageWriter<DialogueRequestFailedEvent>,
) {
    // Poll all tasks and collect completed ones
    let mut i = 0;
    while i < pending_tasks.tasks.len() {
        if let Some((_request_id, original_request, result, mut attempts)) =
            block_on(poll_once(&mut pending_tasks.tasks[i]))
        {
            // Task completed - remove and drop it
            drop(pending_tasks.tasks.swap_remove(i));

            // Handle result
            match result {
                Ok(response) => {
                    limits.record_success(original_request.speaker, &config);
                    response_writer.write(DialogueResponseEvent { response });
                }
                Err(err) => {
                    attempts = attempts.saturating_add(1);
                    match err.kind {
                        DialogueErrorKind::RateLimited {
                            retry_after_seconds,
                        } => {
                            limits.apply_backoff(original_request.speaker, retry_after_seconds);
                        }
                        DialogueErrorKind::ProviderFailure { .. }
                        | DialogueErrorKind::ContextMissing { .. } => {
                            limits.apply_backoff(
                                original_request.speaker,
                                config.retry_backoff_seconds,
                            );
                        }
                    }

                    if attempts <= config.max_retries {
                        // Re-queue the original request with backoff
                        queue.enqueue_with_cooldown(original_request, config.retry_backoff_seconds);
                    } else {
                        failure_writer.write(DialogueRequestFailedEvent { error: err });
                    }
                }
            }
        } else {
            // Task still pending
            i += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::types::{DialogueContext, DialogueRequest, DialogueTopicHint};
    use crate::npc::components::NpcId;

    #[test]
    fn queue_reports_ready_state() {
        let mut queue = DialogueRequestQueue::default();
        assert!(queue.is_empty());

        let request = DialogueRequest::new(
            NpcId::new(1),
            None,
            "Hello",
            DialogueTopicHint::Status,
            DialogueContext::default(),
        );
        let request_id = queue.enqueue(request);

        assert_eq!(request_id.value(), 0);
        assert!(!queue.is_empty());
        assert!(queue.front_ready());

        // Tick the queue and ensure it remains ready with zero cooldown.
        queue.tick(0.5);
        assert!(queue.front_ready());
    }
}
