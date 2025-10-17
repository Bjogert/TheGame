//! Economy-specific events used by the micro trade loop.
use bevy::prelude::{Event, Message};

use crate::{
    economy::{
        components::{Profession, TradeGood},
        dependency::DependencyCategory,
    },
    npc::components::NpcId,
};

#[derive(Event, Message, Debug, Clone)]
pub struct TradeCompletedEvent {
    pub day: u64,
    pub from: Option<NpcId>,
    pub to: Option<NpcId>,
    pub good: TradeGood,
    pub quantity: u32,
    pub reason: TradeReason,
}

/// Snapshot recording whether a profession satisfied dependency categories for a day.
#[derive(Event, Message, Debug, Clone)]
pub struct ProfessionDependencyUpdateEvent {
    pub day: u64,
    pub npc: NpcId,
    pub profession: Profession,
    pub satisfied_categories: Vec<DependencyCategory>,
    pub missing_categories: Vec<DependencyCategory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeReason {
    Production,
    Processing,
    Exchange,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::economy::{components::Profession, dependency::EconomyDependencyMatrix};
    use crate::npc::components::NpcId;

    #[test]
    fn trade_event_exposes_fields() {
        let event = TradeCompletedEvent {
            day: 5,
            from: Some(NpcId::new(1)),
            to: Some(NpcId::new(2)),
            good: TradeGood::Flour,
            quantity: 12,
            reason: TradeReason::Processing,
        };

        assert_eq!(event.day, 5);
        assert_eq!(event.quantity, 12);
        assert_eq!(event.good, TradeGood::Flour);
        assert!(matches!(event.reason, TradeReason::Processing));
        assert_eq!(event.from.unwrap().to_string(), "NPC-0001");
        assert_eq!(event.to.unwrap().to_string(), "NPC-0002");
    }

    #[test]
    fn dependency_update_lists_categories() {
        let matrix = EconomyDependencyMatrix::default();
        let categories = matrix.categories_for_good(TradeGood::Grain).to_vec();
        let event = ProfessionDependencyUpdateEvent {
            day: 8,
            npc: NpcId::new(12),
            profession: Profession::Farmer,
            satisfied_categories: categories.clone(),
            missing_categories: vec![DependencyCategory::Tools],
        };

        assert_eq!(event.day, 8);
        assert_eq!(event.npc.to_string(), "NPC-0012");
        assert_eq!(event.satisfied_categories, categories);
        assert_eq!(event.missing_categories, vec![DependencyCategory::Tools]);
    }
}
