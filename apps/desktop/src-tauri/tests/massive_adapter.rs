//! Wiremock-backed unit tests for `MassiveProvider`.
//!
//! Every test mounts a `wiremock::Mock` serving a captured JSON fixture from
//! `tests/fixtures/massive/`, then drives the adapter through one of the
//! `DataProvider` / `MacroProvider` methods and asserts the decoded shape.
//! No real network access is required.

use std::fs;
use std::path::PathBuf;

use chrdfin_desktop_lib::{
    http::AppHttpClient,
    secrets::MassiveTier,
    sync::{
        error::ProviderError,
        massive::client::MassiveProvider,
        provider::{DataProvider, MacroProvider},
        types::{DividendType, MacroSeriesId, SplitType},
    },
};
use chrono::NaiveDate;
use wiremock::matchers::{header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const TEST_KEY: &str = "test_massive_key";

fn fixture(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/massive")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("fixture {path:?}: {e}"))
}

async fn provider_for(server: &MockServer) -> MassiveProvider {
    MassiveProvider::with_base_url(
        AppHttpClient::new(),
        Some(TEST_KEY.to_string()),
        MassiveTier::Paid, // bypass the 5 RPM bottleneck for fast tests
        server.uri(),
    )
}

// ---------- aggregates -------------------------------------------------------

#[tokio::test]
async fn fetch_prices_happy_path_returns_eight_bars() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v2/aggs/ticker/SPY/range/1/day/2025-04-01/2025-04-10",
        ))
        .and(header(
            "authorization",
            format!("Bearer {TEST_KEY}").as_str(),
        ))
        .and(query_param("adjusted", "true"))
        .and(query_param("sort", "asc"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(fixture("aggs_spy_2025-04-01_2025-04-10.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let prices = provider
        .fetch_prices(
            "SPY",
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
        )
        .await
        .expect("fetch_prices should succeed");

    assert_eq!(prices.len(), 8, "SPY 8-day window");
    let first = &prices[0];
    assert_eq!(first.ticker, "SPY");
    assert_eq!(first.date, NaiveDate::from_ymd_opt(2025, 4, 1).unwrap());
    assert!((first.open - 557.45).abs() < 1e-6, "open mismatch");
    assert!((first.close - 560.97).abs() < 1e-6, "close mismatch");
    assert_eq!(
        first.adj_close, first.close,
        "adj_close should mirror close when adjusted=true"
    );
    assert_eq!(first.volume, 54_609_641);
}

#[tokio::test]
async fn fetch_prices_handles_missing_results_array() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path(
            "/v2/aggs/ticker/SPY/range/1/day/2025-04-01/2025-04-10",
        ))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(
                r#"{"status":"OK","ticker":"SPY","queryCount":0,"resultsCount":0}"#,
            ),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let prices = provider
        .fetch_prices(
            "SPY",
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
        )
        .await
        .expect("empty result is not an error");
    assert!(prices.is_empty());
}

// ---------- ticker reference -------------------------------------------------

#[tokio::test]
async fn search_tickers_happy_path_returns_five_hits() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers"))
        .and(header(
            "authorization",
            format!("Bearer {TEST_KEY}").as_str(),
        ))
        .and(query_param("search", "apple"))
        .and(query_param("active", "true"))
        .and(query_param("limit", "5"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("tickers_search_apple.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let hits = provider
        .search_tickers("apple", 5)
        .await
        .expect("search should succeed");

    assert_eq!(hits.len(), 5);
    let aapl = hits.iter().find(|h| h.ticker == "AAPL").unwrap();
    assert_eq!(aapl.name, "Apple Inc.");
    assert_eq!(aapl.exchange.as_deref(), Some("XNAS"));
}

#[tokio::test]
async fn fetch_metadata_happy_path() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/AAPL"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("ticker_overview_aapl.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let metadata = provider
        .fetch_metadata("AAPL")
        .await
        .expect("fetch_metadata should succeed");

    assert_eq!(metadata.ticker, "AAPL");
    assert_eq!(metadata.name, "Apple Inc.");
    assert!(metadata.is_active);
    assert!(metadata.market_cap.is_some_and(|m| m > 0));
    // SIC description maps to industry; sector stays None until a SIC→sector
    // map lands in Phase 7.
    assert!(metadata.industry.is_some());
    assert!(metadata.sector.is_none());
}

#[tokio::test]
async fn fetch_metadata_404_maps_to_not_found() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/ZZZZZNOTREAL"))
        .respond_with(
            ResponseTemplate::new(404).set_body_string(fixture("ticker_overview_404.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let err = provider
        .fetch_metadata("ZZZZZNOTREAL")
        .await
        .expect_err("404 should propagate");
    assert!(matches!(err, ProviderError::NotFound(_)), "got {err:?}");
}

// ---------- corporate actions ------------------------------------------------

#[tokio::test]
async fn fetch_dividends_classifies_recurring_distribution() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/stocks/v1/dividends"))
        .and(query_param("ticker", "AAPL"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("dividends_aapl_2024.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let dividends = provider
        .fetch_dividends(
            "AAPL",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
        )
        .await
        .expect("dividends should succeed");

    assert_eq!(dividends.len(), 4, "AAPL 2024 paid 4 dividends");
    assert!(
        dividends
            .iter()
            .all(|d| matches!(d.div_type, DividendType::Regular)),
        "all 2024 AAPL dividends are recurring"
    );
    // Three quarters @ $0.25 + one quarter @ $0.24 = $0.99 across the year.
    let total: f64 = dividends.iter().map(|d| d.amount).sum();
    assert!((total - 0.99).abs() < 1e-6, "total dividends: {total}");
}

#[tokio::test]
async fn fetch_splits_classifies_all_three_types() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/stocks/v1/splits"))
        .and(query_param("ticker", "AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("splits_aapl_all.json")))
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let splits = provider
        .fetch_splits(
            "AAPL",
            NaiveDate::from_ymd_opt(2000, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
        )
        .await
        .expect("splits should succeed");

    assert_eq!(splits.len(), 3, "AAPL has had 3 splits in this fixture");
    let s2020 = splits
        .iter()
        .find(|s| s.execution_date == NaiveDate::from_ymd_opt(2020, 8, 31).unwrap())
        .unwrap();
    assert_eq!(s2020.split_from, 1.0);
    assert_eq!(s2020.split_to, 4.0);
    assert!(matches!(s2020.adjustment_type, SplitType::ForwardSplit));
}

// ---------- macro bundles ----------------------------------------------------

#[tokio::test]
async fn fetch_series_treasury_3mo_explodes_bundle_rows() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/fed/v1/treasury-yields"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("treasury_yields_2025-04.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let obs = provider
        .fetch_series(
            MacroSeriesId::Treasury3Mo,
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
        )
        .await
        .expect("treasury 3mo should succeed");

    assert_eq!(obs.len(), 8, "8 weekday observations");
    assert!(obs.iter().all(|o| o.series == MacroSeriesId::Treasury3Mo));
    let first = &obs[0];
    assert_eq!(first.date, NaiveDate::from_ymd_opt(2025, 4, 1).unwrap());
    assert!((first.value - 4.32).abs() < 1e-6, "got {}", first.value);
}

#[tokio::test]
async fn fetch_series_treasury_10y_uses_same_fixture_different_field() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/fed/v1/treasury-yields"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("treasury_yields_2025-04.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let obs = provider
        .fetch_series(
            MacroSeriesId::Treasury10Y,
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
        )
        .await
        .expect("treasury 10y should succeed");

    assert_eq!(obs.len(), 8);
    assert!((obs[0].value - 4.17).abs() < 1e-6);
}

#[tokio::test]
async fn fetch_series_cpi_yoy() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/fed/v1/inflation"))
        .respond_with(ResponseTemplate::new(200).set_body_string(fixture("inflation_2025-Q1.json")))
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let obs = provider
        .fetch_series(
            MacroSeriesId::CpiYoy,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
        )
        .await
        .expect("CPI YoY should succeed");

    assert_eq!(obs.len(), 4);
    assert!(obs.iter().all(|o| o.series == MacroSeriesId::CpiYoy));
    // Sanity: 2025 inflation prints were in the 2-3% range.
    assert!(obs.iter().all(|o| o.value > 0.0 && o.value < 10.0));
}

#[tokio::test]
async fn fetch_series_unemployment_rate() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/fed/v1/labor-market"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("labor_market_2025-Q1.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let obs = provider
        .fetch_series(
            MacroSeriesId::UnemploymentRate,
            NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
        )
        .await
        .expect("unemployment should succeed");

    assert_eq!(obs.len(), 4);
    assert!((obs[0].value - 4.0).abs() < 1e-6);
}

// ---------- error mapping ----------------------------------------------------

#[tokio::test]
async fn rate_limited_response_maps_with_retry_after() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/AAPL"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "12")
                .set_body_string(fixture("rate_limit_429.json")),
        )
        .mount(&server)
        .await;

    let provider = provider_for(&server).await;
    let err = provider
        .fetch_metadata("AAPL")
        .await
        .expect_err("429 should propagate");
    match err {
        ProviderError::RateLimited {
            provider,
            retry_after,
        } => {
            assert_eq!(provider, "massive");
            assert_eq!(retry_after.unwrap().as_secs(), 12);
        }
        other => panic!("expected RateLimited, got {other:?}"),
    }
}

#[tokio::test]
async fn missing_api_key_surfaces_at_request_time() {
    let server = MockServer::start().await;
    // No mock mounted — the request shouldn't even be sent.

    let provider = MassiveProvider::with_base_url(
        AppHttpClient::new(),
        None, // no key configured
        MassiveTier::Free,
        server.uri(),
    );

    let err = provider
        .fetch_metadata("AAPL")
        .await
        .expect_err("missing key should propagate");
    assert!(
        matches!(err, ProviderError::MissingApiKey(provider) if provider == "massive"),
        "got {err:?}"
    );
}
