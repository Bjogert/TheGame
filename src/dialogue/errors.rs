//! Error types surfaced by the dialogue request runner.
use std::fmt;

use super::{broker::DialogueProviderKind, types::DialogueRequestId};

/// Error categories returned when processing dialogue requests.
#[derive(Debug, Clone)]
pub enum DialogueErrorKind {
    RateLimited { retry_after_seconds: f32 },
    ProviderFailure { message: String },
    ContextMissing { missing: DialogueContextSource },
}

impl DialogueErrorKind {
    pub fn rate_limited(retry_after_seconds: f32) -> Self {
        Self::RateLimited {
            retry_after_seconds,
        }
    }

    pub fn provider_failure(message: impl Into<String>) -> Self {
        Self::ProviderFailure {
            message: message.into(),
        }
    }

    pub fn context_missing(missing: DialogueContextSource) -> Self {
        Self::ContextMissing { missing }
    }
}

impl fmt::Display for DialogueErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RateLimited {
                retry_after_seconds,
            } => write!(f, "Rate limited. Retry after {:.2}s", retry_after_seconds),
            Self::ProviderFailure { message } => write!(f, "Provider failure: {}", message),
            Self::ContextMissing { missing } => {
                write!(f, "Missing context: {}", missing)
            }
        }
    }
}

/// Context sources that can cause provider rejections when missing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DialogueContextSource {
    TradeHistory,
    ScheduleState,
    InventoryState,
}

impl fmt::Display for DialogueContextSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Self::TradeHistory => "trade history",
            Self::ScheduleState => "schedule state",
            Self::InventoryState => "inventory state",
        };
        write!(f, "{}", label)
    }
}

/// Full error with provider metadata and request id.
#[derive(Debug, Clone)]
pub struct DialogueError {
    pub request_id: DialogueRequestId,
    pub provider: DialogueProviderKind,
    pub kind: DialogueErrorKind,
}

impl DialogueError {
    pub fn new(
        request_id: DialogueRequestId,
        provider: DialogueProviderKind,
        kind: DialogueErrorKind,
    ) -> Self {
        Self {
            request_id,
            provider,
            kind,
        }
    }
}

impl fmt::Display for DialogueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Dialogue error ({} - request {}): {}",
            self.provider,
            self.request_id.value(),
            self.kind
        )
    }
}

impl std::error::Error for DialogueError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_error_variants() {
        let rate_limited = DialogueErrorKind::rate_limited(2.5);
        match rate_limited {
            DialogueErrorKind::RateLimited {
                retry_after_seconds,
            } => assert_eq!(retry_after_seconds, 2.5),
            _ => panic!("expected rate limited variant"),
        }

        let provider_failure = DialogueErrorKind::provider_failure("unreachable");
        assert!(matches!(
            provider_failure,
            DialogueErrorKind::ProviderFailure { .. }
        ));

        let missing_schedule =
            DialogueErrorKind::context_missing(DialogueContextSource::ScheduleState);
        let missing_inventory =
            DialogueErrorKind::context_missing(DialogueContextSource::InventoryState);

        for missing in [&missing_schedule, &missing_inventory] {
            if let DialogueErrorKind::ContextMissing { missing } = missing {
                // Ensure Display implementation is exercised.
                assert!(
                    missing.to_string().contains("state")
                        || missing.to_string().contains("history")
                );
            } else {
                panic!("expected context missing variant");
            }
        }

        let request_id = DialogueRequestId::new(7);
        assert_eq!(request_id.value(), 7);

        let error = DialogueError::new(
            request_id,
            DialogueProviderKind::OpenAi,
            provider_failure.clone(),
        );
        assert!(error.to_string().contains("OpenAi"));
        assert_eq!(format!("{}", error.kind), format!("{}", provider_failure));
    }
}
