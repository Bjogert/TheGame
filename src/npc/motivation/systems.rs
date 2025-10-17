use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    core::plugin::SimulationClock,
    dialogue::events::DialogueResponseEvent,
    economy::{
        components::Profession,
        dependency::EconomyDependencyMatrix,
        events::{ProfessionDependencyUpdateEvent, TradeCompletedEvent, TradeReason},
    },
    npc::{
        components::{Identity, NpcId},
        events::NpcActivityChangedEvent,
    },
    world::time::WorldClock,
};

use super::{
    config::MotivationConfig,
    state::{adjusted_task_reward, DailyDependencyTracker, NpcMotivation},
};

pub fn reward_from_leisure(
    mut events: MessageReader<NpcActivityChangedEvent>,
    config: Res<MotivationConfig>,
    mut query: Query<(&Identity, &mut NpcMotivation)>,
) {
    #[derive(Default)]
    struct Adjustment {
        leisure: bool,
        alcohol: bool,
        last_time_of_day: Option<f32>,
    }

    let mut adjustments: HashMap<NpcId, Adjustment> = HashMap::new();
    for event in events.read() {
        let activity_lower = event.activity.to_ascii_lowercase();
        let entry = adjustments.entry(event.npc).or_default();
        entry.last_time_of_day = Some(event.time_of_day);

        if config
            .leisure
            .keywords
            .iter()
            .any(|keyword| activity_lower.contains(keyword))
        {
            entry.leisure = true;
        }

        if config
            .alcohol
            .trigger_keywords
            .iter()
            .any(|keyword| activity_lower.contains(keyword))
        {
            entry.alcohol = true;
        }
    }

    for (identity, mut motivation) in query.iter_mut() {
        if let Some(adjustment) = adjustments.get(&identity.id) {
            if adjustment.leisure {
                motivation.apply_reward(config.gains.leisure, &config);
                let mood_label = motivation.mood().label();
                if let Some(fraction) = adjustment.last_time_of_day {
                    info!(
                        "{} enjoys downtime near day fraction {:.2} and feels {}",
                        identity.display_name,
                        fraction,
                        mood_label
                    );
                } else {
                    info!(
                        "{} enjoys downtime and feels {}",
                        identity.display_name, mood_label
                    );
                }
            }

            if adjustment.alcohol {
                motivation.trigger_alcohol_boost(&config);
                let mood_label = motivation.mood().label();
                if let Some(fraction) = adjustment.last_time_of_day {
                    info!(
                        "{} indulges in a drink near day fraction {:.2} and now feels {}",
                        identity.display_name,
                        fraction,
                        mood_label
                    );
                } else {
                    info!(
                        "{} indulges in a drink and now feels {}",
                        identity.display_name, mood_label
                    );
                }
            }
        }
    }
}

pub fn reward_from_trade_events(
    mut trades: MessageReader<TradeCompletedEvent>,
    config: Res<MotivationConfig>,
    mut query: Query<(&Identity, &mut NpcMotivation)>,
) {
    let mut rewards: HashMap<NpcId, f32> = HashMap::new();
    for event in trades.read() {
        let actor = event.from.or(event.to);
        if let Some(actor) = actor {
            let reward = match event.reason {
                TradeReason::Production | TradeReason::Processing => config.gains.task,
                TradeReason::Exchange => config.gains.task * 0.5,
            };
            *rewards.entry(actor).or_insert(0.0) += reward;
        }
    }

    for (identity, mut motivation) in query.iter_mut() {
        if let Some(amount) = rewards.remove(&identity.id) {
            let adjusted = adjusted_task_reward(amount, &config.alcohol, &motivation);
            motivation.apply_reward(adjusted, &config);
            info!(
                "{} completes trade work and gains {:.1} motivation",
                identity.display_name, adjusted
            );
        }
    }
}

pub fn reward_from_dialogue_responses(
    mut responses: MessageReader<DialogueResponseEvent>,
    config: Res<MotivationConfig>,
    mut query: Query<(&Identity, &mut NpcMotivation)>,
) {
    let mut rewards: HashMap<NpcId, f32> = HashMap::new();
    for event in responses.read() {
        let speaker = event.response.speaker;
        *rewards.entry(speaker).or_insert(0.0) += config.gains.social;

        if let Some(target) = event.response.target {
            *rewards.entry(target).or_insert(0.0) += config.gains.social * 0.6;
        }
    }

    for (identity, mut motivation) in query.iter_mut() {
        if let Some(amount) = rewards.remove(&identity.id) {
            motivation.apply_reward(amount, &config);
            debug!(
                "{} feels uplifted after conversation (+{:.1})",
                identity.display_name, amount
            );
        }
    }
}

pub fn track_dependency_satisfaction(
    mut updates: MessageReader<ProfessionDependencyUpdateEvent>,
    mut tracker: ResMut<DailyDependencyTracker>,
) {
    for update in updates.read() {
        tracker.prepare_day(update.day);
        for category in &update.satisfied_categories {
            tracker.record(update.day, update.npc, *category);
        }

        if !update.missing_categories.is_empty() {
            debug!(
                "{} missing {} dependencies on day {}",
                update.profession.label(),
                update
                    .missing_categories
                    .iter()
                    .map(|category| category.label())
                    .collect::<Vec<_>>()
                    .join(", "),
                update.day
            );
        }
    }
}

pub fn evaluate_dependency_impacts(
    clock: Res<WorldClock>,
    matrix: Res<EconomyDependencyMatrix>,
    config: Res<MotivationConfig>,
    mut tracker: ResMut<DailyDependencyTracker>,
    mut query: Query<(&Identity, &Profession, &mut NpcMotivation)>,
) {
    let current_day = clock.day_count();
    let Some(evaluated_day) = tracker.next_ready_day(current_day) else {
        return;
    };

    let satisfied_map = tracker.take_satisfied_for_day(evaluated_day);
    for (identity, profession, mut motivation) in query.iter_mut() {
        let requirements = matrix.requirements(*profession);
        if requirements.is_empty() {
            continue;
        }

        let flags = satisfied_map.get(&identity.id);
        let mut missing = 0;
        for category in requirements {
            let met = flags.map_or(false, |entry| entry.contains(*category));
            if met {
                continue;
            }

            missing += 1;
            motivation.apply_penalty(config.dependency.deficit_penalty, &config);
            warn!(
                "{} lacks {} support on day {}",
                identity.display_name,
                category.label(),
                evaluated_day
            );
        }

        if missing == 0 {
            motivation.apply_reward(config.dependency.satisfaction_bonus, &config);
            debug!(
                "{} satisfied wellbeing dependencies for day {}",
                identity.display_name, evaluated_day
            );
        }
    }
}

pub fn decay_npc_motivation(
    sim_clock: Res<SimulationClock>,
    config: Res<MotivationConfig>,
    mut query: Query<(&Identity, &mut NpcMotivation)>,
) {
    let delta = sim_clock.last_scaled_delta().as_secs_f32();
    if delta <= 0.0 {
        return;
    }

    for (identity, mut motivation) in query.iter_mut() {
        let outcome = motivation.tick(delta, &config);
        if let Some(mood) = outcome.mood_changed {
            info!(
                "{} mood shifts to {} (dopamine {:.1})",
                identity.display_name,
                mood.label(),
                motivation.dopamine()
            );
        }

        if outcome.hangover_triggered {
            warn!("{} enters a hangover crash", identity.display_name);
        }
    }
}
