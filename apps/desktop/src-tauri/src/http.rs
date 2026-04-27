use std::time::Duration;

use reqwest::redirect::Policy;

/// Shared HTTP client for every outbound provider request. Constructed once
/// at app startup and stored on `AppState`. Wrapping `reqwest::Client` in a
/// newtype keeps adapter signatures honest (you can't accidentally hand them
/// an unrelated client) and gives us a single seam to swap during tests.
#[derive(Debug, Clone)]
pub struct AppHttpClient {
    inner: reqwest::Client,
}

impl AppHttpClient {
    pub fn new() -> Self {
        // unwrap is acceptable at startup — failure here means rustls/system
        // TLS is unusable, in which case the app cannot function anyway.
        let inner = reqwest::Client::builder()
            .user_agent(concat!(
                "chrdfin/",
                env!("CARGO_PKG_VERSION"),
                " (+https://github.com/chris-c-thomas/chrdfin)"
            ))
            .timeout(Duration::from_secs(30))
            .redirect(Policy::limited(5))
            .gzip(true)
            .build()
            .expect("failed to build reqwest client");
        Self { inner }
    }

    /// Borrow the underlying `reqwest::Client`. Adapters (sub-phase 1B)
    /// use this directly; no consumers exist yet in 1A.
    #[allow(dead_code)]
    pub fn inner(&self) -> &reqwest::Client {
        &self.inner
    }
}

impl Default for AppHttpClient {
    fn default() -> Self {
        Self::new()
    }
}
