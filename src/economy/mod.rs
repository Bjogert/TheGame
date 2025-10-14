//! Economy module hosting placeholder trade loops and resource definitions.
pub mod components;
pub mod events;
pub mod plugin;
pub mod resources;
pub mod systems;

pub use components::{Inventory, Profession, TradeGood};
pub use events::{TradeCompletedEvent, TradeReason};
pub use plugin::EconomyPlugin;
