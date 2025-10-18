use bevy::prelude::{debug, MessageWriter};

use crate::dialogue::{
    queue::DialogueRequestQueue,
    types::{
        DialogueContext, DialogueContextEvent, DialogueRequest, DialogueTopicHint, TradeContext,
        TradeContextReason, TradeDescriptor,
    },
};
use crate::npc::components::NpcId;

use super::super::{
    components::TradeGood,
    events::{TradeCompletedEvent, TradeReason},
};

const TRADE_PROMPT_VERB: &str = "discusses exchanging a";
const SCHEDULE_PROMPT_ACTION: &str = "reviews the day's schedule";
const SCHEDULE_SUMMARY_PREFIX: &str = "Daily plan:";
const SENTENCE_SUFFIX: &str = ".";

pub(super) struct TradeDialogueInput {
    pub(super) day: u64,
    pub(super) from: Option<NpcId>,
    pub(super) to: Option<NpcId>,
    pub(super) good: TradeGood,
    pub(super) quantity: u32,
    pub(super) reason: TradeReason,
}

pub(super) fn queue_schedule_brief(
    queue: &mut DialogueRequestQueue,
    day: u64,
    speaker: NpcId,
    description: String,
) {
    let mut context =
        DialogueContext::with_events(vec![DialogueContextEvent::ScheduleUpdate { description }]);
    context.summary = Some(format!("{SCHEDULE_SUMMARY_PREFIX} Day {day}"));

    let prompt = format!(
        "{speaker} {action}{suffix}",
        speaker = speaker,
        action = SCHEDULE_PROMPT_ACTION,
        suffix = SENTENCE_SUFFIX
    );

    let request = DialogueRequest::new(speaker, None, prompt, DialogueTopicHint::Schedule, context);
    let id = queue.enqueue(request);
    debug!(
        "Queued schedule update dialogue {} for speaker {} on day {}",
        id.value(),
        speaker,
        day
    );
}

pub(super) fn send_trade_and_dialogue(
    trade_writer: &mut MessageWriter<TradeCompletedEvent>,
    queue: &mut DialogueRequestQueue,
    input: TradeDialogueInput,
) {
    trade_writer.write(TradeCompletedEvent {
        day: input.day,
        from: input.from,
        to: input.to,
        good: input.good,
        quantity: input.quantity,
        reason: input.reason,
    });

    if let (Some(speaker), Some(target)) = (input.from, input.to) {
        let descriptor = TradeDescriptor::new(input.good.label(), input.quantity);
        let context =
            DialogueContext::with_events(vec![DialogueContextEvent::Trade(TradeContext {
                day: input.day,
                from: input.from,
                to: input.to,
                descriptor,
                reason: input.reason.into(),
            })]);
        let prompt = build_trade_prompt(speaker, input.good.label());
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

fn build_trade_prompt(speaker: NpcId, good_label: &str) -> String {
    format!(
        "{speaker} {verb} {good}{suffix}",
        speaker = speaker,
        verb = TRADE_PROMPT_VERB,
        good = good_label,
        suffix = SENTENCE_SUFFIX
    )
}
