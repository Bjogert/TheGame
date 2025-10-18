use std::collections::{BTreeMap, HashMap};

use bevy::prelude::*;

use crate::economy::dependency::DependencyCategory;

use super::config::{AlcoholConfig, MotivationConfig};
use crate::npc::components::NpcId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NpcMood {
    Energised,
    Content,
    Tired,
    Depressed,
}

impl NpcMood {
    pub fn label(self) -> &'static str {
        match self {
            Self::Energised => "energised",
            Self::Content => "content",
            Self::Tired => "tired",
            Self::Depressed => "depressed",
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct NpcMotivation {
    dopamine: f32,
    mood: NpcMood,
    intoxication_timer: f32,
    hangover_timer: f32,
}

impl NpcMotivation {
    pub fn new(config: &MotivationConfig) -> Self {
        let mut motivation = Self {
            dopamine: config.defaults.start,
            mood: NpcMood::Content,
            intoxication_timer: 0.0,
            hangover_timer: 0.0,
        };
        motivation.recompute_mood(config);
        motivation
    }

    pub fn dopamine(&self) -> f32 {
        self.dopamine
    }

    pub fn mood(&self) -> NpcMood {
        self.mood
    }

    pub fn is_intoxicated(&self) -> bool {
        self.intoxication_timer > 0.0
    }

    pub fn is_in_hangover(&self) -> bool {
        self.hangover_timer > 0.0
    }

    pub fn apply_reward(&mut self, amount: f32, config: &MotivationConfig) {
        if amount <= 0.0 {
            return;
        }
        self.dopamine = (self.dopamine + amount).min(config.defaults.max);
        self.recompute_mood(config);
    }

    pub fn apply_penalty(&mut self, amount: f32, config: &MotivationConfig) {
        if amount <= 0.0 {
            return;
        }
        self.dopamine = (self.dopamine - amount).max(config.defaults.min);
        self.recompute_mood(config);
    }

    pub fn trigger_alcohol_boost(&mut self, config: &MotivationConfig) {
        self.apply_reward(config.alcohol.boost, config);
        self.intoxication_timer = config.alcohol.intoxication_seconds;
    }

    pub fn tick(&mut self, delta_seconds: f32, config: &MotivationConfig) -> MotivationTickOutcome {
        let mut outcome = MotivationTickOutcome::default();
        if delta_seconds <= 0.0 {
            return outcome;
        }

        let mut decay_amount = config.decay.per_second * delta_seconds;
        if self.hangover_timer > 0.0 {
            self.hangover_timer = (self.hangover_timer - delta_seconds).max(0.0);
            decay_amount *= config.alcohol.hangover_decay_multiplier;
        }

        self.apply_penalty(decay_amount, config);

        if self.intoxication_timer > 0.0 {
            let previous = self.intoxication_timer;
            self.intoxication_timer = (self.intoxication_timer - delta_seconds).max(0.0);
            if previous > 0.0 && self.intoxication_timer == 0.0 {
                self.apply_penalty(config.alcohol.hangover_penalty, config);
                self.hangover_timer = config.alcohol.hangover_duration_seconds;
                outcome.hangover_triggered = true;
            }
        }

        let new_mood = determine_mood(self.dopamine, config);
        if new_mood != self.mood {
            self.mood = new_mood;
            outcome.mood_changed = Some(new_mood);
        }

        outcome
    }

    fn recompute_mood(&mut self, config: &MotivationConfig) {
        self.mood = determine_mood(self.dopamine, config);
    }
}

#[derive(Default, Debug, Clone)]
pub struct MotivationTickOutcome {
    pub mood_changed: Option<NpcMood>,
    pub hangover_triggered: bool,
}

fn determine_mood(level: f32, config: &MotivationConfig) -> NpcMood {
    if level >= config.thresholds.energised {
        NpcMood::Energised
    } else if level >= config.thresholds.content {
        NpcMood::Content
    } else if level >= config.thresholds.tired {
        NpcMood::Tired
    } else {
        NpcMood::Depressed
    }
}

#[derive(Resource, Debug, Default)]
pub struct DailyDependencyTracker {
    satisfied_by_day: BTreeMap<u64, HashMap<NpcId, CategoryFlags>>,
}

impl DailyDependencyTracker {
    pub fn prepare_day(&mut self, day: u64) {
        self.satisfied_by_day.entry(day).or_default();
    }

    pub fn record(&mut self, day: u64, npc: NpcId, category: DependencyCategory) {
        self.prepare_day(day);
        if let Some(entries) = self.satisfied_by_day.get_mut(&day) {
            entries
                .entry(npc)
                .or_insert_with(CategoryFlags::default)
                .insert(category);
        }
    }

    pub fn next_ready_day(&self, current_day: u64) -> Option<u64> {
        self.satisfied_by_day
            .keys()
            .copied()
            .find(|tracked_day| *tracked_day < current_day)
    }

    pub fn take_satisfied_for_day(&mut self, day: u64) -> HashMap<NpcId, CategoryFlags> {
        self.satisfied_by_day.remove(&day).unwrap_or_default()
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct CategoryFlags(u8);

impl CategoryFlags {
    fn insert(&mut self, category: DependencyCategory) {
        self.0 |= 1 << (category as u8);
    }

    pub fn contains(&self, category: DependencyCategory) -> bool {
        (self.0 & (1 << (category as u8))) != 0
    }
}

pub fn adjusted_task_reward(
    amount: f32,
    alcohol: &AlcoholConfig,
    motivation: &NpcMotivation,
) -> f32 {
    if (motivation.is_intoxicated() || motivation.is_in_hangover()) && alcohol.quality_penalty > 0.0
    {
        amount * (1.0 - alcohol.quality_penalty)
    } else {
        amount
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motivation_tick_updates_mood() {
        let config = MotivationConfig::load_or_default();
        let mut motivation = NpcMotivation::new(&config);
        motivation.apply_penalty(50.0, &config);
        assert_eq!(motivation.mood(), NpcMood::Depressed);
        motivation.apply_reward(80.0, &config);
        assert_eq!(motivation.mood(), NpcMood::Energised);
    }

    #[test]
    fn dependency_tracker_records_flags() {
        let mut tracker = DailyDependencyTracker::default();
        let npc = NpcId::new(1);
        tracker.record(0, npc, DependencyCategory::Food);
        let day = tracker.next_ready_day(1).expect("day should be ready");
        assert_eq!(day, 0);
        let satisfied = tracker.take_satisfied_for_day(day);
        assert!(satisfied[&npc].contains(DependencyCategory::Food));
    }
}
