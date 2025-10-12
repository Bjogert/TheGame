//! Dialogue request queue and rate-limiting logic.
use std::{
    collections::{HashMap, VecDeque},
    time::Duration,
};

use bevy::prelude::*;

use crate::npc::components::NpcId;

use super::broker::{DialogueError, DialogueProvider, DialogueRequestId};

#[derive(Debug, Clone)]
pub struct DialogueRequest {
    pub id: DialogueRequestId,
    pub provider: DialogueProvider,
    pub npc_id: Option<NpcId>,
    pub prompt: String,
    pub retries: u8,
}

impl DialogueRequest {
    pub fn new(
        id: DialogueRequestId,
        provider: DialogueProvider,
        npc_id: Option<NpcId>,
        prompt: impl Into<String>,
    ) -> Self {
        Self {
            id,
            provider,
            npc_id,
            prompt: prompt.into(),
            retries: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DialogueQueueConfig {
    pub global_cooldown_secs: f32,
    pub per_npc_cooldown_secs: f32,
    pub max_retries: u8,
    pub max_dispatches_per_tick: usize,
}

impl Default for DialogueQueueConfig {
    fn default() -> Self {
        Self {
            global_cooldown_secs: 1.0,
            per_npc_cooldown_secs: 30.0,
            max_retries: 3,
            max_dispatches_per_tick: 4,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct DialogueQueueMetrics {
    pub enqueued: u64,
    pub accepted: u64,
    pub deferred: u64,
    pub retries_scheduled: u64,
    pub throttled: u64,
    pub timed_out: u64,
    pub transport_errors: u64,
    pub dropped: u64,
}

impl DialogueQueueMetrics {
    pub fn queue_depth(&self, pending: usize, delayed: usize) -> usize {
        pending + delayed
    }
}

#[derive(Debug)]
struct CooldownTracker {
    remaining: f32,
}

impl CooldownTracker {
    fn ready() -> Self {
        Self { remaining: 0.0 }
    }

    fn tick(&mut self, delta_seconds: f32) {
        self.remaining = (self.remaining - delta_seconds).max(0.0);
    }

    fn is_ready(&self) -> bool {
        self.remaining <= f32::EPSILON
    }

    fn trigger(&mut self, cooldown: f32) {
        self.remaining = cooldown.max(0.0);
    }
}

#[derive(Debug)]
struct RetryEntry {
    request: DialogueRequest,
    remaining: f32,
}

impl RetryEntry {
    fn new(request: DialogueRequest, delay: Duration) -> Self {
        Self {
            request,
            remaining: delay.as_secs_f32().max(0.0),
        }
    }
}

#[derive(Resource)]
pub struct DialogueRequestQueue {
    pending: VecDeque<DialogueRequest>,
    delayed: Vec<RetryEntry>,
    npc_cooldowns: HashMap<NpcId, CooldownTracker>,
    global_cooldown: CooldownTracker,
    config: DialogueQueueConfig,
    metrics: DialogueQueueMetrics,
    next_id: u64,
}

impl Default for DialogueRequestQueue {
    fn default() -> Self {
        Self::new(DialogueQueueConfig::default())
    }
}

impl DialogueRequestQueue {
    pub fn new(config: DialogueQueueConfig) -> Self {
        Self {
            pending: VecDeque::new(),
            delayed: Vec::new(),
            npc_cooldowns: HashMap::new(),
            global_cooldown: CooldownTracker::ready(),
            config,
            metrics: DialogueQueueMetrics::default(),
            next_id: 1,
        }
    }

    pub fn config(&self) -> DialogueQueueConfig {
        self.config
    }

    pub fn metrics(&self) -> &DialogueQueueMetrics {
        &self.metrics
    }

    pub fn queue_depth(&self) -> usize {
        self.pending.len() + self.delayed.len()
    }

    pub fn enqueue(
        &mut self,
        provider: DialogueProvider,
        npc_id: Option<NpcId>,
        prompt: impl Into<String>,
    ) -> DialogueRequestId {
        let id = DialogueRequestId::new(self.next_id);
        self.next_id += 1;
        let request = DialogueRequest::new(id, provider, npc_id, prompt);
        self.pending.push_back(request);
        self.metrics.enqueued += 1;
        id
    }

    pub fn take_ready(&mut self) -> Vec<DialogueRequest> {
        self.take_ready_limit(self.config.max_dispatches_per_tick)
    }

    pub fn take_ready_limit(&mut self, limit: usize) -> Vec<DialogueRequest> {
        if limit == 0 {
            return Vec::new();
        }

        let mut ready = Vec::new();
        for _ in 0..limit {
            if !self.global_cooldown.is_ready() {
                break;
            }

            let pending_len = self.pending.len();
            if pending_len == 0 {
                break;
            }

            let mut dispatched: Option<DialogueRequest> = None;
            for _ in 0..pending_len {
                if let Some(request) = self.pending.pop_front() {
                    if self.can_dispatch(&request) {
                        self.global_cooldown
                            .trigger(self.config.global_cooldown_secs);
                        if let Some(npc) = request.npc_id {
                            self.npc_cooldowns
                                .entry(npc)
                                .or_insert_with(CooldownTracker::ready)
                                .trigger(self.config.per_npc_cooldown_secs);
                        }
                        dispatched = Some(request);
                        break;
                    } else {
                        self.pending.push_back(request);
                    }
                }
            }

            match dispatched {
                Some(request) => ready.push(request),
                None => break,
            }
        }

        ready
    }

    pub fn tick(&mut self, delta_seconds: f32) {
        self.global_cooldown.tick(delta_seconds);
        for tracker in self.npc_cooldowns.values_mut() {
            tracker.tick(delta_seconds);
        }

        let mut still_delayed = Vec::with_capacity(self.delayed.len());
        for mut entry in self.delayed.drain(..) {
            entry.remaining = (entry.remaining - delta_seconds).max(0.0);
            if entry.remaining <= f32::EPSILON {
                self.pending.push_back(entry.request);
            } else {
                still_delayed.push(entry);
            }
        }
        self.delayed = still_delayed;
    }

    pub fn record_accept(&mut self, _request: &DialogueRequest, _latency: Duration) {
        self.metrics.accepted += 1;
    }

    pub fn record_deferred(&mut self, request: DialogueRequest, retry_after: Duration) {
        self.metrics.deferred += 1;
        self.schedule_retry(request, retry_after, false);
    }

    pub fn record_error(&mut self, request: DialogueRequest, error: DialogueError) {
        match error {
            DialogueError::Throttled { retry_after, .. } => {
                self.metrics.throttled += 1;
                self.schedule_retry(request, retry_after, false);
            }
            DialogueError::Timeout { .. } => {
                self.metrics.timed_out += 1;
                self.retry_or_drop(request, Duration::from_secs_f32(1.0));
            }
            DialogueError::Transport { .. } => {
                self.metrics.transport_errors += 1;
                self.retry_or_drop(request, Duration::from_secs_f32(2.0));
            }
            DialogueError::UnsupportedProvider { provider, .. } => {
                self.metrics.dropped += 1;
                warn!(
                    target: "dialogue",
                    "Dropping request for unsupported provider {provider}; depth={}",
                    self.queue_depth()
                );
            }
            DialogueError::Cancelled { reason, .. } => {
                self.metrics.dropped += 1;
                warn!(
                    target: "dialogue",
                    "Cancelled request removed from queue: {reason}; depth={}",
                    self.queue_depth()
                );
            }
        }
    }

    fn retry_or_drop(&mut self, request: DialogueRequest, delay: Duration) {
        self.schedule_retry(request, delay, true);
    }

    fn schedule_retry(
        &mut self,
        mut request: DialogueRequest,
        delay: Duration,
        increment_retry: bool,
    ) {
        if increment_retry {
            if request.retries >= self.config.max_retries {
                self.metrics.dropped += 1;
                warn!(
                    target: "dialogue",
                    "Dropping request {} after exhausting retries (max_retries={}); depth={}",
                    request.id,
                    self.config.max_retries,
                    self.queue_depth()
                );
                return;
            }
            request.retries = request.retries.saturating_add(1);
        }

        if delay.is_zero() {
            self.pending.push_back(request);
        } else {
            self.delayed.push(RetryEntry::new(request, delay));
        }
        self.metrics.retries_scheduled += 1;
    }

    fn can_dispatch(&self, request: &DialogueRequest) -> bool {
        if let Some(npc) = request.npc_id {
            self.npc_cooldowns
                .get(&npc)
                .map(|tracker| tracker.is_ready())
                .unwrap_or(true)
        } else {
            true
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_queue(global: f32, per_npc: f32, max_retries: u8) -> DialogueRequestQueue {
        DialogueRequestQueue::new(DialogueQueueConfig {
            global_cooldown_secs: global,
            per_npc_cooldown_secs: per_npc,
            max_retries,
            max_dispatches_per_tick: 8,
        })
    }

    #[test]
    fn enforces_global_rate_limit() {
        let mut queue = make_queue(1.0, 0.0, 2);
        queue.enqueue(DialogueProvider::Local, None, "first");
        queue.enqueue(DialogueProvider::Local, None, "second");

        let first_batch = queue.take_ready_limit(8);
        assert_eq!(first_batch.len(), 1);

        queue.tick(0.5);
        assert!(queue.take_ready_limit(8).is_empty());

        queue.tick(0.6);
        let second_batch = queue.take_ready_limit(8);
        assert_eq!(second_batch.len(), 1);
    }

    #[test]
    fn retries_drop_after_limit() {
        let mut queue = make_queue(0.0, 0.0, 1);
        queue.enqueue(DialogueProvider::Local, None, "hi");

        let mut request = queue.take_ready_limit(1).pop().unwrap();
        queue.record_error(
            request.clone(),
            DialogueError::Timeout {
                request_id: request.id,
                elapsed: Duration::from_secs(1),
            },
        );

        queue.tick(1.0);
        request = queue.take_ready_limit(1).pop().unwrap();
        queue.record_error(
            request.clone(),
            DialogueError::Timeout {
                request_id: request.id,
                elapsed: Duration::from_secs(1),
            },
        );

        queue.tick(1.0);
        assert!(queue.take_ready_limit(1).is_empty());
        assert_eq!(queue.metrics().dropped, 1);
    }
}
