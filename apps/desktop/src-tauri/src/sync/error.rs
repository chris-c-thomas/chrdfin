use std::time::Duration;

/// Shared error type for every data-provider adapter.
///
/// Variants are deliberately coarse — adapters classify their HTTP results
/// into these buckets and the orchestrator (sub-phase 1D) decides what to do
/// with each one (retry, skip, abort, surface to the user).
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    /// The credential the adapter needs is missing from `Secrets`. Surfaces
    /// when a sync is attempted without `MASSIVE_API_KEY` configured.
    #[error("missing api key for provider {0}")]
    MissingApiKey(&'static str),

    /// The provider returned 429. The orchestrator honors `retry_after` when
    /// present; otherwise it falls back to its own backoff schedule.
    #[error("rate limited by provider {provider} — retry after {retry_after:?}")]
    RateLimited {
        provider: &'static str,
        retry_after: Option<Duration>,
    },

    /// Any underlying transport-level failure (DNS, TLS, connection reset,
    /// timeout, body decode at the bytes layer).
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    /// The provider returned a non-success status that wasn't a 404 or 429.
    /// Includes the response body for diagnostics.
    #[error("provider returned status {status} for {url}: {body}")]
    BadStatus {
        status: u16,
        url: String,
        body: String,
    },

    /// JSON parsing or shape-validation failure for a successful response.
    #[error("response decode error: {0}")]
    Decode(String),

    /// 404 from a ticker-specific endpoint (or empty result from one that
    /// promises a single resource). Distinct from `BadStatus` so on-demand
    /// fetch can convert this into a user-friendly "ticker not found".
    #[error("ticker not found: {0}")]
    NotFound(String),

    /// The free tier's monthly cap (or another hard quota limit) has been
    /// reached. The orchestrator finalizes the run with a warning rather
    /// than treating this as an abort.
    #[error("free-tier limit reached: {0}")]
    FreeTierLimit(&'static str),
}

pub type ProviderResult<T> = Result<T, ProviderError>;
