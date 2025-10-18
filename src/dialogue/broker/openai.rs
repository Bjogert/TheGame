use bevy::log::warn;
use reqwest::{
    blocking::Client,
    header::{HeaderMap, RETRY_AFTER},
    StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::dialogue::types::{
    DialogueContextEvent, DialogueRequest, DialogueRequestId, DialogueResponse, DialogueTopicHint,
    TradeContextReason,
};
use crate::npc::components::NpcId;

use super::super::errors::{DialogueContextSource, DialogueError, DialogueErrorKind};
use super::{
    config::{OpenAiConfig, OpenAiConfigError},
    DialogueBroker, DialogueProviderKind,
};

const EMPTY_PROMPT_ERROR: &str = "prompt cannot be empty";
const MANUAL_RETRY_PROMPT: &str = "retry later";
const MANUAL_RETRY_BACKOFF_SECONDS: f32 = 3.0;
const FALLBACK_TARGET_LABEL: &str = "player";
const SUMMARY_PREFIX: &str = "Summary:";
const SCHEDULE_UPDATE_PREFIX: &str = "Schedule update:";
const CONTEXT_FALLBACK_MESSAGE: &str = "No notable context available.";
const SENTENCE_SUFFIX: &str = ".";
const DEFAULT_RATE_LIMIT_BACKOFF: f32 = 10.0;
const USER_MESSAGE_SPEAKER_PREFIX: &str = "Speaker: ";
const USER_MESSAGE_TARGET_PREFIX: &str = "Target: ";
const USER_MESSAGE_TOPIC_PREFIX: &str = "Topic: ";
const USER_MESSAGE_PROMPT_PREFIX: &str = "Prompt: ";
const USER_MESSAGE_CONTEXT_SUMMARY_PREFIX: &str = "Context summary: ";
const USER_MESSAGE_RESPONSE_INSTRUCTION: &str =
    "Respond as the speaker, addressing the target naturally.";
const USER_MESSAGE_TRADE_EVENT_PREFIX: &str = "Trade event: Day ";
const USER_MESSAGE_TRADE_FROM_PREFIX: &str = " (from ";
const USER_MESSAGE_TRADE_TO_PREFIX: &str = " (to ";
const USER_MESSAGE_WITH_SUFFIX: &str = " with ";
const USER_MESSAGE_FROM_SUFFIX: &str = " after receiving it from ";
const USER_MESSAGE_TRADE_SUFFIX: &str = ")";
const TRADE_DETAIL_DAY_PREFIX: &str = "On day ";
const TRADE_DETAIL_THEY_PREFIX: &str = " they ";
const SYSTEM_PROMPT: &str = "You are a medieval villager in a life-simulation game. Respond briefly (1-3 sentences), stay in character, and reference only the supplied context. If information is missing, acknowledge the gap.";

/// Primary OpenAI dialogue broker.
pub struct OpenAiDialogueBroker {
    mode: BrokerMode,
}

enum BrokerMode {
    Live(OpenAiLiveClient),
    Fallback,
}

impl OpenAiDialogueBroker {
    pub fn new() -> Self {
        match OpenAiConfig::from_env() {
            Ok(config) => match OpenAiLiveClient::new(config) {
                Ok(client) => Self {
                    mode: BrokerMode::Live(client),
                },
                Err(err) => {
                    warn!(
                        "OpenAI broker running in fallback mode ({}). Check HTTP client configuration.",
                        err
                    );
                    Self {
                        mode: BrokerMode::Fallback,
                    }
                }
            },
            Err(OpenAiConfigError::MissingApiKey) => {
                warn!("OPENAI_API_KEY not set; dialogue broker using local fallback responses.");
                Self {
                    mode: BrokerMode::Fallback,
                }
            }
            Err(OpenAiConfigError::ClientBuild(message)) => {
                warn!(
                    "Failed to construct OpenAI HTTP client ({}). Falling back to local responses.",
                    message
                );
                Self {
                    mode: BrokerMode::Fallback,
                }
            }
        }
    }

    fn validate(&self, request: &DialogueRequest) -> Result<(), DialogueErrorKind> {
        if request.prompt.trim().is_empty() {
            return Err(DialogueErrorKind::provider_failure(EMPTY_PROMPT_ERROR));
        }

        if request.prompt.eq_ignore_ascii_case(MANUAL_RETRY_PROMPT) {
            return Err(DialogueErrorKind::rate_limited(
                MANUAL_RETRY_BACKOFF_SECONDS,
            ));
        }

        match request.topic_hint {
            DialogueTopicHint::Trade => {
                if request.context.summary.is_none() {
                    return Err(DialogueErrorKind::context_missing(
                        DialogueContextSource::InventoryState,
                    ));
                }

                if !request
                    .context
                    .events
                    .iter()
                    .any(|event| matches!(event, DialogueContextEvent::Trade(_)))
                {
                    return Err(DialogueErrorKind::context_missing(
                        DialogueContextSource::TradeHistory,
                    ));
                }
            }
            DialogueTopicHint::Schedule => {
                if !request
                    .context
                    .events
                    .iter()
                    .any(|event| matches!(event, DialogueContextEvent::ScheduleUpdate { .. }))
                {
                    return Err(DialogueErrorKind::context_missing(
                        DialogueContextSource::ScheduleState,
                    ));
                }
            }
            DialogueTopicHint::Status => {}
        }

        Ok(())
    }

    fn fabricate_response(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> DialogueResponse {
        let content = compose_context_segments(request);
        DialogueResponse::new(
            request_id,
            self.provider_kind(),
            request.speaker,
            request.target,
            content,
        )
    }
}

impl DialogueBroker for OpenAiDialogueBroker {
    fn provider_kind(&self) -> DialogueProviderKind {
        DialogueProviderKind::OpenAi
    }

    fn process(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> Result<DialogueResponse, DialogueError> {
        if let Err(kind) = self.validate(request) {
            return Err(DialogueError::new(request_id, self.provider_kind(), kind));
        }

        match &self.mode {
            BrokerMode::Live(client) => match client.send(request_id, request) {
                Ok(response) => Ok(response),
                Err(kind) => Err(DialogueError::new(request_id, self.provider_kind(), kind)),
            },
            BrokerMode::Fallback => Ok(self.fabricate_response(request_id, request)),
        }
    }
}

struct OpenAiLiveClient {
    http: Client,
    config: OpenAiConfig,
}

impl OpenAiLiveClient {
    fn new(config: OpenAiConfig) -> Result<Self, OpenAiConfigError> {
        let http = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|err| OpenAiConfigError::ClientBuild(err.to_string()))?;

        Ok(Self { http, config })
    }

    fn send(
        &self,
        request_id: DialogueRequestId,
        request: &DialogueRequest,
    ) -> Result<DialogueResponse, DialogueErrorKind> {
        let payload = ChatCompletionRequest {
            model: self.config.model.as_str(),
            messages: build_messages(request),
            max_tokens: Some(self.config.max_output_tokens.into()),
            temperature: self.config.temperature,
        };

        let url = self.config.chat_url();
        let response = self
            .http
            .post(url)
            .bearer_auth(&self.config.api_key)
            .json(&payload)
            .send()
            .map_err(|err| DialogueErrorKind::provider_failure(err.to_string()))?;

        let status = response.status();
        let headers = response.headers().clone();

        if status == StatusCode::TOO_MANY_REQUESTS {
            let retry_after = parse_retry_after(&headers).unwrap_or(DEFAULT_RATE_LIMIT_BACKOFF);
            return Err(DialogueErrorKind::rate_limited(retry_after));
        }

        if !status.is_success() {
            if let Ok(body) = response.json::<OpenAiErrorResponse>() {
                let message = format!(
                    "{} (type: {}, code: {:?})",
                    body.error.message, body.error.error_type, body.error.code
                );
                return Err(DialogueErrorKind::provider_failure(message));
            }

            return Err(DialogueErrorKind::provider_failure(format!(
                "HTTP {} from OpenAI",
                status
            )));
        }

        let completion: ChatCompletionResponse = response
            .json()
            .map_err(|err| DialogueErrorKind::provider_failure(err.to_string()))?;

        let content = completion
            .choices
            .into_iter()
            .find_map(|choice| choice.message.content)
            .map(|text| text.trim().to_string())
            .filter(|text| !text.is_empty())
            .ok_or_else(|| {
                DialogueErrorKind::provider_failure(
                    "OpenAI returned an empty completion for dialogue request",
                )
            })?;

        Ok(DialogueResponse::new(
            request_id,
            DialogueProviderKind::OpenAi,
            request.speaker,
            request.target,
            content,
        ))
    }
}

fn parse_retry_after(headers: &HeaderMap) -> Option<f32> {
    headers.get(RETRY_AFTER).and_then(|value| {
        value
            .to_str()
            .ok()
            .and_then(|text| text.parse::<f32>().ok())
    })
}

fn build_messages(request: &DialogueRequest) -> Vec<ChatMessage> {
    let mut messages = Vec::new();
    messages.push(ChatMessage {
        role: "system",
        content: SYSTEM_PROMPT.to_string(),
    });

    messages.push(ChatMessage {
        role: "user",
        content: build_user_message(request),
    });

    messages
}

fn build_user_message(request: &DialogueRequest) -> String {
    let mut sections = Vec::new();
    sections.push(format!("{USER_MESSAGE_SPEAKER_PREFIX}{}", request.speaker));
    let target = request
        .target
        .map(|id| id.to_string())
        .unwrap_or_else(|| FALLBACK_TARGET_LABEL.to_string());
    sections.push(format!("{USER_MESSAGE_TARGET_PREFIX}{}", target));
    sections.push(format!(
        "{USER_MESSAGE_TOPIC_PREFIX}{}",
        topic_label(request.topic_hint)
    ));
    sections.push(format!(
        "{USER_MESSAGE_PROMPT_PREFIX}{}",
        request.prompt.trim()
    ));

    if let Some(summary) = &request.context.summary {
        if !summary.trim().is_empty() {
            sections.push(format!(
                "{USER_MESSAGE_CONTEXT_SUMMARY_PREFIX}{}",
                summary.trim()
            ));
        }
    }

    for event in &request.context.events {
        match event {
            DialogueContextEvent::Trade(trade) => {
                let action = match trade.reason {
                    TradeContextReason::Production => "produced",
                    TradeContextReason::Processing => "processed",
                    TradeContextReason::Exchange => "exchanged",
                };
                let mut detail = format!(
                    "{USER_MESSAGE_TRADE_EVENT_PREFIX}{} {} {} {}",
                    trade.day, action, trade.descriptor.quantity, trade.descriptor.label
                );
                if let Some(from) = trade.from {
                    detail.push_str(&format!(
                        "{USER_MESSAGE_TRADE_FROM_PREFIX}{}{USER_MESSAGE_TRADE_SUFFIX}",
                        from
                    ));
                }
                if let Some(to) = trade.to {
                    detail.push_str(&format!(
                        "{USER_MESSAGE_TRADE_TO_PREFIX}{}{USER_MESSAGE_TRADE_SUFFIX}",
                        to
                    ));
                }
                sections.push(detail);
            }
            DialogueContextEvent::ScheduleUpdate { description } => {
                if !description.trim().is_empty() {
                    sections.push(format!("{SCHEDULE_UPDATE_PREFIX} {}", description.trim()));
                }
            }
        }
    }

    if sections.len() == 4 {
        sections.push(CONTEXT_FALLBACK_MESSAGE.to_string());
    }

    sections.push(USER_MESSAGE_RESPONSE_INSTRUCTION.to_string());
    sections.join("\n")
}

fn compose_context_segments(request: &DialogueRequest) -> String {
    let mut segments = Vec::new();
    segments.push(request.prompt.trim().to_string());

    if let Some(summary) = &request.context.summary {
        if !summary.trim().is_empty() {
            segments.push(format!("{} {}", SUMMARY_PREFIX, summary.trim()));
        }
    }

    let target_label = request
        .target
        .map(|id| id.to_string())
        .unwrap_or_else(|| FALLBACK_TARGET_LABEL.to_string());
    segments.push(format!("{USER_MESSAGE_TARGET_PREFIX}{}", target_label));

    for event in &request.context.events {
        match event {
            DialogueContextEvent::Trade(trade) => {
                let quantity = trade.descriptor.quantity;
                let label = &trade.descriptor.label;
                let action = match trade.reason {
                    TradeContextReason::Production => "produced",
                    TradeContextReason::Processing => "processed",
                    TradeContextReason::Exchange => "exchanged",
                };
                let mut detail = format!(
                    "{TRADE_DETAIL_DAY_PREFIX}{}{TRADE_DETAIL_THEY_PREFIX}{} {} {}",
                    trade.day, action, quantity, label
                );
                if let Some(target) = trade.to {
                    detail.push_str(&format!("{USER_MESSAGE_WITH_SUFFIX}{}", target));
                }
                if let Some(source) = trade.from {
                    detail.push_str(&format!("{USER_MESSAGE_FROM_SUFFIX}{}", source));
                }
                detail.push_str(SENTENCE_SUFFIX);
                segments.push(detail);
            }
            DialogueContextEvent::ScheduleUpdate { description } => {
                segments.push(format!(
                    "{SCHEDULE_UPDATE_PREFIX} {}{SENTENCE_SUFFIX}",
                    description
                ));
            }
        }
    }

    if segments.is_empty() {
        segments.push(CONTEXT_FALLBACK_MESSAGE.to_string());
    }

    segments.join(" ")
}

fn topic_label(topic: DialogueTopicHint) -> &'static str {
    match topic {
        DialogueTopicHint::Status => "status",
        DialogueTopicHint::Trade => "trade",
        DialogueTopicHint::Schedule => "schedule",
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
    #[serde(rename = "max_tokens")]
    max_tokens: Option<u32>,
    temperature: f32,
}

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatChoiceMessage,
}

#[derive(Debug, Deserialize)]
struct ChatChoiceMessage {
    content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorResponse {
    error: OpenAiErrorDetail,
}

#[derive(Debug, Deserialize)]
struct OpenAiErrorDetail {
    message: String,
    #[serde(rename = "type")]
    error_type: String,
    code: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dialogue::types::{
        DialogueContext, DialogueTopicHint, TradeContext, TradeDescriptor,
    };

    #[test]
    fn fallback_response_includes_context() {
        let broker = OpenAiDialogueBroker {
            mode: BrokerMode::Fallback,
        };

        let trade_context = DialogueContextEvent::Trade(TradeContext {
            day: 3,
            from: Some(NpcId::new(1)),
            to: Some(NpcId::new(2)),
            descriptor: TradeDescriptor::new("grain crate", 2),
            reason: TradeContextReason::Exchange,
        });

        let request = DialogueRequest::new(
            NpcId::new(1),
            Some(NpcId::new(2)),
            "Discuss the latest trade",
            DialogueTopicHint::Trade,
            DialogueContext {
                summary: Some("Short summary".to_string()),
                events: vec![trade_context],
            },
        );

        let response = broker
            .process(DialogueRequestId::new(7), &request)
            .expect("fallback should succeed");
        assert!(response.content.contains("Summary"));
        assert!(response.content.contains("grain crate"));
        assert_eq!(response.provider, DialogueProviderKind::OpenAi);
    }

    #[test]
    fn manual_retry_prompt_triggers_backoff() {
        let broker = OpenAiDialogueBroker {
            mode: BrokerMode::Fallback,
        };

        let request = DialogueRequest::new(
            NpcId::new(1),
            None,
            MANUAL_RETRY_PROMPT,
            DialogueTopicHint::Status,
            DialogueContext::default(),
        );

        let error = broker
            .process(DialogueRequestId::new(1), &request)
            .expect_err("retry prompt should error");
        assert!(matches!(error.kind, DialogueErrorKind::RateLimited { .. }));
    }
}
