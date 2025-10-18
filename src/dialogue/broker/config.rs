use std::{env, fmt, time::Duration};

const DEFAULT_BASE_URL: &str = "https://api.openai.com";
const DEFAULT_CHAT_PATH: &str = "/v1/chat/completions";
const DEFAULT_MODEL: &str = "gpt-4o-mini";
const DEFAULT_TEMPERATURE: f32 = 0.7;
const DEFAULT_MAX_OUTPUT_TOKENS: u16 = 220;
const DEFAULT_TIMEOUT_SECS: u64 = 15;

/// OpenAI chat configuration sourced from the environment.
#[derive(Debug, Clone)]
pub struct OpenAiConfig {
    pub api_key: String,
    pub base_url: String,
    pub model: String,
    pub max_output_tokens: u16,
    pub temperature: f32,
    pub timeout: Duration,
}

impl OpenAiConfig {
    pub fn from_env() -> Result<Self, OpenAiConfigError> {
        let api_key = env::var("OPENAI_API_KEY")
            .map_err(|_| OpenAiConfigError::MissingApiKey)
            .and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    Err(OpenAiConfigError::MissingApiKey)
                } else {
                    Ok(trimmed.to_string())
                }
            })?;

        let base_url = env::var("OPENAI_BASE_URL")
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|_| DEFAULT_BASE_URL.to_string());

        let model = env::var("OPENAI_MODEL")
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        let timeout = env::var("OPENAI_TIMEOUT_SECS")
            .ok()
            .and_then(|value| value.parse::<u64>().ok())
            .filter(|value| *value > 0)
            .map(Duration::from_secs)
            .unwrap_or_else(|| Duration::from_secs(DEFAULT_TIMEOUT_SECS));

        let max_output_tokens = env::var("OPENAI_MAX_OUTPUT_TOKENS")
            .ok()
            .and_then(|value| value.parse::<u16>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_MAX_OUTPUT_TOKENS);

        let temperature = env::var("OPENAI_TEMPERATURE")
            .ok()
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| *value >= 0.0)
            .unwrap_or(DEFAULT_TEMPERATURE);

        Ok(Self {
            api_key,
            base_url,
            model,
            max_output_tokens,
            temperature,
            timeout,
        })
    }

    pub fn chat_url(&self) -> String {
        format!(
            "{}{}",
            self.base_url.trim_end_matches('/'),
            DEFAULT_CHAT_PATH
        )
    }
}

#[derive(Debug)]
pub enum OpenAiConfigError {
    MissingApiKey,
    ClientBuild(String),
}

impl fmt::Display for OpenAiConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingApiKey => write!(f, "missing OPENAI_API_KEY"),
            Self::ClientBuild(message) => write!(f, "client build failure: {}", message),
        }
    }
}

impl std::error::Error for OpenAiConfigError {}
