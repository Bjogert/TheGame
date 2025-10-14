//! Dialogue request queue and rate limiting resources.
use std::collections::HashMap;
use std::collections::VecDeque;

use bevy::prelude::*;

use crate::npc::components::NpcId;

use super::{
    broker::{DialogueBroker, DialogueProviderKind},
    errors::{DialogueError, DialogueErrorKind},
    events::{DialogueRequestFailedEvent, DialogueResponseEvent},
    types::{DialogueRequest, DialogueRequestId},
};

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
            global_cooldown_seconds: 1.5,
            per_npc_cooldown_seconds: 8.0,
            max_retries: 2,
            retry_backoff_seconds: 5.0,
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
        match self.npc_remaining.get(&speaker) {
            Some(value) if *value > 0.0 => false,
            _ => true,
        }
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
#[derive(Resource)]
pub struct ActiveDialogueBroker {
    inner: Box<dyn DialogueBroker>,
}

impl ActiveDialogueBroker {
    pub fn new(inner: Box<dyn DialogueBroker>) -> Self {
        Self { inner }
    }

    pub fn provider_kind(&self) -> DialogueProviderKind {
        self.inner.provider_kind()
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
    let delta = time.delta_seconds().max(0.0);
    queue.tick(delta);
    limits.tick(delta);
}

/// Processes a single dialogue request if rate limits allow.
pub fn run_dialogue_request_queue(
    mut queue: ResMut<DialogueRequestQueue>,
    mut limits: ResMut<DialogueRateLimitState>,
    config: Res<DialogueRateLimitConfig>,
    broker: Res<ActiveDialogueBroker>,
    mut response_writer: EventWriter<DialogueResponseEvent>,
    mut failure_writer: EventWriter<DialogueRequestFailedEvent>,
) {
    if !queue.front_ready() {
        return;
    }

    let Some(mut queued) = queue.pending.pop_front() else {
        return;
    };

    if !limits.can_process(queued.request.speaker) {
        queue.pending.push_front(queued);
        return;
    }

    match broker.process(queued.id, &queued.request) {
        Ok(response) => {
            limits.record_success(queued.request.speaker, &config);
            response_writer.send(DialogueResponseEvent { response });
        }
        Err(err) => {
            queued.attempts = queued.attempts.saturating_add(1);
            match err.kind {
                DialogueErrorKind::RateLimited {
                    retry_after_seconds,
                } => {
                    limits.apply_backoff(queued.request.speaker, retry_after_seconds);
                }
                DialogueErrorKind::ProviderFailure { .. }
                | DialogueErrorKind::ContextMissing { .. } => {
                    limits.apply_backoff(queued.request.speaker, config.retry_backoff_seconds);
                }
            }

            if queued.attempts <= config.max_retries {
                queued.cooldown_remaining = config.retry_backoff_seconds;
                queue.pending.push_back(queued);
            } else {
                failure_writer.send(DialogueRequestFailedEvent { error: err });
            }
        }
    }
}
