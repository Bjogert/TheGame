//! Economy module hosting placeholder trade loops and resource definitions.
pub mod components;
pub mod data;
pub mod dependency;
pub mod events;
pub mod planning;
pub mod plugin;
pub mod resources;
pub mod systems;
pub mod tasks;

pub use plugin::EconomyPlugin;

#[cfg(test)]
mod tests {
    use super::{
        components::{Inventory, Profession, TradeGood},
        events::{TradeCompletedEvent, TradeReason},
    };
    use crate::npc::components::NpcId;

    #[test]
    fn reexports_are_available() {
        let mut inventory = Inventory::default();
        inventory.add_good(TradeGood::Grain, 6);
        assert_eq!(inventory.quantity_of(TradeGood::Grain), 6);

        let event = TradeCompletedEvent {
            day: 2,
            from: Some(NpcId::new(1)),
            to: None,
            good: TradeGood::Grain,
            quantity: 6,
            reason: TradeReason::Production,
        };
        assert!(matches!(event.reason, TradeReason::Production));
        assert_eq!(Profession::Farmer.label(), "farmer");
    }
}
