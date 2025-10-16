//! Economy-specific events used by the micro trade loop.
use bevy::prelude::{Event, Message};

use crate::npc::components::NpcId;

use super::components::TradeGood;

#[derive(Event, Message, Debug, Clone)]
pub struct TradeCompletedEvent {
    pub day: u64,
    pub from: Option<NpcId>,
    pub to: Option<NpcId>,
    pub good: TradeGood,
    pub quantity: u32,
    pub reason: TradeReason,
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
}
