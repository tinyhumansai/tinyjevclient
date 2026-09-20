//! Client configuration and measured result types.

use std::{fmt, time::Duration};

use crate::{Error, EvaluationResponse};

const DEFAULT_SYSTEM_ONE_PATH: &str = "v1/systemone";
const TINYHUMANS_SYSTEM_ONE_PATH: &str = "agent-integrations/openrouter/systemone";

/// System One API provider.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Provider {
    /// `TypeSafe`'s first-party System One API.
    #[default]
    TypeSafe,
    /// `OpenRouter`'s compatible System One API.
    OpenRouter,
}

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
    /// API origin or path prefix without the System One endpoint path.
    pub base_url: String,
    /// Provider-specific System One endpoint path.
    pub(super) system_one_path: &'static str,
    /// Provider-specific response validation behavior.
    pub provider: Provider,
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
            system_one_path: DEFAULT_SYSTEM_ONE_PATH,
            provider: Provider::TypeSafe,
            timeout: Duration::from_secs(30),
            retry: RetryPolicy::default(),
        }
    }

    /// Create configuration for `OpenRouter`'s System One API.
    #[must_use]
    pub fn openrouter(api_key: impl Into<String>) -> Self {
        Self {
            api_key: ApiKey(api_key.into()),
            base_url: "https://openrouter.ai/api".to_owned(),
            system_one_path: DEFAULT_SYSTEM_ONE_PATH,
            provider: Provider::OpenRouter,
            timeout: Duration::from_secs(30),
            retry: RetryPolicy::default(),
        }
    }

    /// Create configuration for the `TinyHumans` `OpenRouter` System One proxy.
    ///
    /// The proxy accepts a `TinyHumans` API key and forwards typed Jev requests
    /// to `OpenRouter` while applying the caller's `TinyHumans` account limits.
    #[must_use]
    pub fn tinyhumans_openrouter(api_key: impl Into<String>) -> Self {
        Self {
            api_key: ApiKey(api_key.into()),
            base_url: "https://api.tinyhumans.ai".to_owned(),
            system_one_path: TINYHUMANS_SYSTEM_ONE_PATH,
            provider: Provider::OpenRouter,
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
            .field("system_one_path", &self.system_one_path)
            .field("provider", &self.provider)
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

/// A failed evaluation with the attempts and elapsed time it spent.
#[derive(Debug, thiserror::Error)]
#[error("{error}")]
pub struct EvaluationFailure {
    /// Classified terminal failure.
    #[source]
    pub error: Error,
    /// HTTP attempts made before failure; zero for local request validation.
    pub attempts: u32,
    /// End-to-end elapsed time including retry delays.
    pub latency: Duration,
}
