//! Opt-in live integration tests against the real `api.massive.com`.
//!
//! These are gated with `#[ignore]` so `cargo test` never runs them and CI
//! never spends API quota. Run manually with:
//!
//! ```sh
//! MASSIVE_API_KEY=... cargo test -p chrdfin-desktop --test massive_live -- --ignored
//! ```
//!
//! Each test exercises one endpoint category and asserts only minimal
//! shape — the wiremock suite (`tests/massive_adapter.rs`) is the
//! authoritative correctness gate. These exist as a smoke check that the
//! real API hasn't drifted from the captured fixtures.

use chrdfin_desktop_lib::{
    http::AppHttpClient,
    secrets::MassiveTier,
    sync::{
        massive::client::MassiveProvider,
        provider::{DataProvider, MacroProvider},
        types::MacroSeriesId,
    },
};
use chrono::NaiveDate;

fn live_provider() -> Option<MassiveProvider> {
    let key = std::env::var("MASSIVE_API_KEY")
        .ok()
        .filter(|k| !k.is_empty());
    key.map(|k| MassiveProvider::new(AppHttpClient::new(), Some(k), MassiveTier::Free))
}

#[tokio::test]
#[ignore = "live API call — opt in with `cargo test -- --ignored`"]
async fn live_fetch_prices_spy_recent_week() {
    let provider = live_provider().expect("MASSIVE_API_KEY must be set for live tests");
    let prices = provider
        .fetch_prices(
            "SPY",
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
        )
        .await
        .expect("live fetch_prices");
    assert!(!prices.is_empty(), "should return at least one bar");
    assert!(prices.iter().all(|p| p.close > 0.0));
}

#[tokio::test]
#[ignore = "live API call — opt in with `cargo test -- --ignored`"]
async fn live_search_tickers_apple() {
    let provider = live_provider().expect("MASSIVE_API_KEY must be set for live tests");
    let hits = provider.search_tickers("apple", 5).await.expect("search");
    assert!(hits.iter().any(|h| h.ticker == "AAPL"));
}

#[tokio::test]
#[ignore = "live API call — opt in with `cargo test -- --ignored`"]
async fn live_fetch_metadata_aapl() {
    let provider = live_provider().expect("MASSIVE_API_KEY must be set for live tests");
    let metadata = provider.fetch_metadata("AAPL").await.expect("metadata");
    assert_eq!(metadata.ticker, "AAPL");
}

#[tokio::test]
#[ignore = "live API call — opt in with `cargo test -- --ignored`"]
async fn live_fetch_dividends_aapl_2024() {
    let provider = live_provider().expect("MASSIVE_API_KEY must be set for live tests");
    let dividends = provider
        .fetch_dividends(
            "AAPL",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        )
        .await
        .expect("dividends");
    assert!(dividends.len() >= 4);
}

#[tokio::test]
#[ignore = "live API call — opt in with `cargo test -- --ignored`"]
async fn live_fetch_treasury_10y() {
    let provider = live_provider().expect("MASSIVE_API_KEY must be set for live tests");
    let obs = provider
        .fetch_series(
            MacroSeriesId::Treasury10Y,
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
        )
        .await
        .expect("treasury 10y");
    assert!(!obs.is_empty());
}
