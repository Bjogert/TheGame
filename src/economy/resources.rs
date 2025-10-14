//! Economy resources for placeholder trade loops.
use bevy::prelude::Resource;

#[derive(Resource, Debug, Default)]
pub struct MicroTradeLoopState {
    pub last_processed_day: Option<u64>,
}
