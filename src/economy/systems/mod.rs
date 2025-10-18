//! Systems powering the config-driven economy planner.

pub mod day_prep;
pub mod dialogue;
pub mod spawning;
pub mod task_execution;

pub use day_prep::prepare_economy_day;
pub use spawning::{assign_placeholder_professions, spawn_profession_crates};
pub use task_execution::advance_actor_tasks;
