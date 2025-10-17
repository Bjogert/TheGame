//! Economy-related components such as professions and inventories.
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Placeholder professions used by the micro trade loop.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Profession {
    Farmer,
    Miller,
    Blacksmith,
}

impl Profession {
    pub fn label(self) -> &'static str {
        match self {
            Self::Farmer => "farmer",
            Self::Miller => "miller",
            Self::Blacksmith => "blacksmith",
        }
    }
}

/// Simplified trade goods used for placeholder exchanges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeGood {
    Grain,
    Flour,
    Tools,
}

impl TradeGood {
    pub fn label(self) -> &'static str {
        match self {
            Self::Grain => "grain crate",
            Self::Flour => "flour crate",
            Self::Tools => "tool crate",
        }
    }
}

/// Marker identifying a crate entity representing a profession's work spot.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ProfessionCrate {
    pub profession: Profession,
}

/// Marker for a spawned placeholder representing a trade good near a profession crate.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TradeGoodPlaceholder {
    pub profession: Profession,
    pub good: TradeGood,
}

/// Inventory storing simple stacks of goods.
#[derive(Component, Debug, Clone, Default)]
pub struct Inventory {
    items: Vec<InventoryItem>,
}

impl Inventory {
    pub fn add_good(&mut self, good: TradeGood, quantity: u32) {
        if quantity == 0 {
            return;
        }
        if let Some(entry) = self.items.iter_mut().find(|entry| entry.good == good) {
            entry.quantity = entry.quantity.saturating_add(quantity);
        } else {
            self.items.push(InventoryItem { good, quantity });
        }
    }

    pub fn remove_good(&mut self, good: TradeGood, quantity: u32) -> bool {
        if quantity == 0 {
            return true;
        }
        if let Some(position) = self
            .items
            .iter()
            .position(|entry| entry.good == good && entry.quantity >= quantity)
        {
            let entry = &mut self.items[position];
            entry.quantity -= quantity;
            if entry.quantity == 0 {
                self.items.remove(position);
            }
            true
        } else {
            false
        }
    }

    pub fn quantity_of(&self, good: TradeGood) -> u32 {
        self.items
            .iter()
            .find(|entry| entry.good == good)
            .map(|entry| entry.quantity)
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone)]
struct InventoryItem {
    good: TradeGood,
    quantity: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_manages_quantities() {
        let mut inventory = Inventory::default();
        assert_eq!(inventory.quantity_of(TradeGood::Grain), 0);

        inventory.add_good(TradeGood::Grain, 5);
        inventory.add_good(TradeGood::Grain, 2);
        inventory.add_good(TradeGood::Tools, 1);

        assert_eq!(inventory.quantity_of(TradeGood::Grain), 7);
        assert!(inventory.remove_good(TradeGood::Grain, 4));
        assert_eq!(inventory.quantity_of(TradeGood::Grain), 3);
        assert!(!inventory.remove_good(TradeGood::Grain, 5));

        assert_eq!(Profession::Farmer.label(), "farmer");
        assert_eq!(TradeGood::Tools.label(), "tool crate");

        let marker = ProfessionCrate {
            profession: Profession::Farmer,
        };
        assert_eq!(marker.profession, Profession::Farmer);
    }
}
