//! Systems powering the placeholder micro trade loop.
use bevy::prelude::*;

use crate::{
    dialogue::{
        DialogueContext, DialogueContextEvent, DialogueRequest, DialogueRequestQueue,
        DialogueTopicHint, TradeContext, TradeContextReason, TradeDescriptor,
    },
    npc::components::{Identity, NpcId},
    world::time::WorldClock,
};

use super::{
    components::{Inventory, Profession, TradeGood},
    events::{TradeCompletedEvent, TradeReason},
    resources::MicroTradeLoopState,
};

/// Assigns placeholder professions and empty inventories to debug NPCs.
pub fn assign_placeholder_professions(
    mut commands: Commands,
    query: Query<(Entity, &Identity), Without<Profession>>,
) {
    for (entity, identity) in query.iter() {
        let profession = match identity.display_name.as_str() {
            "Alric" => Some(Profession::Farmer),
            "Bryn" => Some(Profession::Miller),
            "Cedric" => Some(Profession::Blacksmith),
            _ => None,
        };

        if let Some(profession) = profession {
            info!(
                "Assigning {} as {}",
                identity.display_name,
                profession.label()
            );
            commands
                .entity(entity)
                .insert((profession, Inventory::default()));
        }
    }
}

/// Runs once per in-game day to simulate a simple trade loop between professions.
pub fn process_micro_trade_loop(
    world_clock: Res<WorldClock>,
    mut state: ResMut<MicroTradeLoopState>,
    identity_query: Query<(Entity, &Identity, &Profession)>,
    mut inventories: Query<&mut Inventory>,
    mut trade_writer: EventWriter<TradeCompletedEvent>,
    mut dialogue_queue: ResMut<DialogueRequestQueue>,
) {
    let day = world_clock.day_count();
    if state.last_processed_day == Some(day) {
        return;
    }
    state.last_processed_day = Some(day);

    let mut farmer = None;
    let mut miller = None;
    let mut blacksmith = None;

    for (entity, identity, profession) in identity_query.iter() {
        match profession {
            Profession::Farmer => {
                farmer = Some((entity, identity.id, identity.display_name.clone()))
            }
            Profession::Miller => {
                miller = Some((entity, identity.id, identity.display_name.clone()))
            }
            Profession::Blacksmith => {
                blacksmith = Some((entity, identity.id, identity.display_name.clone()))
            }
        }
    }

    let (farmer_entity, farmer_id, farmer_name) = match farmer {
        Some(data) => data,
        None => {
            warn!("Micro trade loop skipped: no farmer present");
            return;
        }
    };
    let (miller_entity, miller_id, miller_name) = match miller {
        Some(data) => data,
        None => {
            warn!("Micro trade loop skipped: no miller present");
            return;
        }
    };
    let (smith_entity, smith_id, smith_name) = match blacksmith {
        Some(data) => data,
        None => {
            warn!("Micro trade loop skipped: no blacksmith present");
            return;
        }
    };

    let Ok([mut farmer_inv, mut miller_inv, mut smith_inv]) =
        inventories.get_many_mut([farmer_entity, miller_entity, smith_entity])
    else {
        warn!("Micro trade loop skipped: inventory lookup failed");
        return;
    };

    // Farmer produces grain for the day.
    farmer_inv.add_good(TradeGood::GrainCrate, 1);
    trade_writer.send(TradeCompletedEvent {
        day,
        from: None,
        to: Some(farmer_id),
        good: TradeGood::GrainCrate,
        quantity: 1,
        reason: TradeReason::Production,
    });
    info!("{} harvests a grain crate", farmer_name);

    // Farmer delivers grain to the miller.
    if farmer_inv.remove_good(TradeGood::GrainCrate, 1) {
        miller_inv.add_good(TradeGood::GrainCrate, 1);
        send_trade_and_dialogue(
            &mut trade_writer,
            &mut dialogue_queue,
            day,
            Some(farmer_id),
            Some(miller_id),
            TradeGood::GrainCrate,
            1,
            TradeReason::Exchange,
        );
        info!("{} passes grain crate to {}", farmer_name, miller_name);
    } else {
        warn!("{} has no grain crate to trade", farmer_name);
        return;
    }

    // Miller processes grain into flour.
    if miller_inv.remove_good(TradeGood::GrainCrate, 1) {
        miller_inv.add_good(TradeGood::FlourCrate, 1);
        trade_writer.send(TradeCompletedEvent {
            day,
            from: Some(miller_id),
            to: Some(miller_id),
            good: TradeGood::FlourCrate,
            quantity: 1,
            reason: TradeReason::Processing,
        });
    } else {
        warn!("{} missing grain crate for milling", miller_name);
        return;
    }

    // Miller delivers flour to the blacksmith.
    if miller_inv.remove_good(TradeGood::FlourCrate, 1) {
        smith_inv.add_good(TradeGood::FlourCrate, 1);
        send_trade_and_dialogue(
            &mut trade_writer,
            &mut dialogue_queue,
            day,
            Some(miller_id),
            Some(smith_id),
            TradeGood::FlourCrate,
            1,
            TradeReason::Exchange,
        );
        info!("{} sends flour crate to {}", miller_name, smith_name);
    } else {
        warn!("{} missing flour crate for delivery", miller_name);
        return;
    }

    // Blacksmith processes flour into tool crate (placeholder transformation).
    if smith_inv.remove_good(TradeGood::FlourCrate, 1) {
        smith_inv.add_good(TradeGood::ToolCrate, 1);
        trade_writer.send(TradeCompletedEvent {
            day,
            from: Some(smith_id),
            to: Some(smith_id),
            good: TradeGood::ToolCrate,
            quantity: 1,
            reason: TradeReason::Processing,
        });
    } else {
        warn!("{} missing flour crate to craft tools", smith_name);
        return;
    }

    // Blacksmith returns tools to the farmer.
    if smith_inv.remove_good(TradeGood::ToolCrate, 1) {
        farmer_inv.add_good(TradeGood::ToolCrate, 1);
        send_trade_and_dialogue(
            &mut trade_writer,
            &mut dialogue_queue,
            day,
            Some(smith_id),
            Some(farmer_id),
            TradeGood::ToolCrate,
            1,
            TradeReason::Exchange,
        );
        info!("{} supplies tool crate to {}", smith_name, farmer_name);
    } else {
        warn!("{} missing tool crate for delivery", smith_name);
    }
}

fn send_trade_and_dialogue(
    trade_writer: &mut EventWriter<TradeCompletedEvent>,
    queue: &mut DialogueRequestQueue,
    day: u64,
    from: Option<NpcId>,
    to: Option<NpcId>,
    good: TradeGood,
    quantity: u32,
    reason: TradeReason,
) {
    trade_writer.send(TradeCompletedEvent {
        day,
        from,
        to,
        good,
        quantity,
        reason,
    });

    if let (Some(speaker), Some(target)) = (from, to) {
        let descriptor = TradeDescriptor::new(good.label(), quantity);
        let context =
            DialogueContext::with_events(vec![DialogueContextEvent::Trade(TradeContext {
                day,
                from,
                to,
                descriptor,
                reason: reason.into(),
            })]);
        let prompt = format!("{} discusses exchanging a {}.", speaker, good.label());
        let request = DialogueRequest::new(
            speaker,
            Some(target),
            prompt,
            DialogueTopicHint::Trade,
            context,
        );
        let id = queue.enqueue(request);
        debug!("Queued dialogue request {} for trade", id.value());
    }
}

impl From<TradeReason> for TradeContextReason {
    fn from(value: TradeReason) -> Self {
        match value {
            TradeReason::Production => TradeContextReason::Production,
            TradeReason::Processing => TradeContextReason::Processing,
            TradeReason::Exchange => TradeContextReason::Exchange,
        }
    }
}
