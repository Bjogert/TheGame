pub mod config;
pub mod state;
pub mod systems;

pub use config::{
    AlcoholConfig, DependencyImpactConfig, LeisureConfig, MotivationConfig, MotivationDecay,
    MotivationDefaults, MotivationGains, MotivationMoodThresholds,
};
pub use state::{DailyDependencyTracker, MotivationTickOutcome, NpcMood, NpcMotivation};
pub use systems::{
    decay_npc_motivation, evaluate_dependency_impacts, reward_from_dialogue_responses,
    reward_from_leisure, reward_from_trade_events, track_dependency_satisfaction,
};
