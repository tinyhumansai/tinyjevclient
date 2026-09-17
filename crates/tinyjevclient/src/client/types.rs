//! Client configuration and measured result types.

use std::{fmt, time::Duration};

use crate::EvaluationResponse;

/// Async `TypeSafe` System One client.
#[derive(Clone)]
pub struct Client {
    pub(super) config: ClientConfig,
    pub(super) http: reqwest::Client,
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

/// HTTP client configuration.
#[derive(Clone)]
pub struct ClientConfig {
    pub(super) api_key: ApiKey,
    /// API root without the versioned endpoint path.
    pub base_url: String,
    /// Total timeout for one HTTP attempt.
    pub timeout: Duration,
    /// Transient failure retry policy.
    pub retry: RetryPolicy,
}

impl ClientConfig {
    /// Create production configuration for an API key.
    #[must_use]
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: ApiKey(api_key.into()),
            base_url: "https://api.typesafe.ai".to_owned(),
            timeout: Duration::from_secs(30),
            retry: RetryPolicy::default(),
        }
    }

    /// Replace the API key without exposing it through a public field.
    #[must_use]
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = ApiKey(api_key.into());
        self
    }
}

impl fmt::Debug for ClientConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientConfig")
            .field("api_key", &"[REDACTED]")
            .field("base_url", &self.base_url)
            .field("timeout", &self.timeout)
            .field("retry", &self.retry)
            .finish()
    }
}

#[derive(Clone)]
pub(super) struct ApiKey(String);

impl ApiKey {
    pub(super) fn expose(&self) -> &str {
        &self.0
    }
}

/// Retry limits for transient failures.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetryPolicy {
    /// Additional attempts after the initial request.
    pub max_retries: u32,
    /// Delay used after the first retryable failure.
    pub initial_backoff: Duration,
    /// Maximum delay, including provider-requested delays.
    pub max_backoff: Duration,
}

impl RetryPolicy {
    pub(super) fn delay(self, attempts: u32) -> Duration {
        let exponent = attempts.saturating_sub(1).min(31);
        self.initial_backoff
            .saturating_mul(1_u32.checked_shl(exponent).unwrap_or(u32::MAX))
            .min(self.max_backoff)
    }
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(2),
        }
    }
}

/// A validated response with transport measurements.
#[derive(Clone, Debug, PartialEq)]
pub struct EvaluationResult {
    /// Typed, validated provider response.
    pub response: EvaluationResponse,
    /// Provider request id, when returned as a header.
    pub request_id: Option<String>,
    /// HTTP attempts including the successful or terminal attempt.
    pub attempts: u32,
    /// End-to-end elapsed time including retry delays.
    pub latency: Duration,
}
