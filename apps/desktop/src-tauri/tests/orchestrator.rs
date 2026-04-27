//! Integration tests for `sync::orchestrator::Sync` against mocked Massive
//! HTTP endpoints + an in-memory DuckDB.
//!
//! The orchestrator is the join point for everything else in the data
//! layer: provider → storage → settings → sync_log. These tests verify
//! the wiring end-to-end without touching the real network or the user's
//! on-disk database file.
//!
//! Tier is pinned to `Paid` so the per-provider rate limiter doesn't
//! throttle the test (Free = 5 RPM). The orchestrator's `history_clamp`
//! still picks 1990-01-01 as the start date on Paid, but the wiremock
//! matchers ignore date specifics via `path_regex`, so the window
//! doesn't matter for assertion correctness.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use chrdfin_desktop_lib::{
    db::Database,
    http::AppHttpClient,
    secrets::MassiveTier,
    storage::{prices, settings, sync_log},
    sync::{
        massive::client::MassiveProvider,
        orchestrator::{SETTING_TRACKED_UNIVERSE, Sync, SyncMode, SyncProgress},
    },
};
use chrono::{NaiveDate, Utc};
use duckdb::Connection;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SCHEMA_SQL: &str = include_str!("../src/schema.sql");

const TICKER_OVERVIEW_AAPL: &str = include_str!("fixtures/massive/ticker_overview_aapl.json");
const TICKER_OVERVIEW_404: &str = include_str!("fixtures/massive/ticker_overview_404.json");
const AGGS_SPY: &str = include_str!("fixtures/massive/aggs_spy_2025-04-01_2025-04-10.json");

/// Empty payloads for endpoints we mount to keep the orchestrator happy
/// but whose contents we don't assert on.
const EMPTY_AGGS: &str =
    r#"{"status":"OK","ticker":"X","queryCount":0,"resultsCount":0,"results":[]}"#;
const EMPTY_PAGED: &str = r#"{"status":"OK","results":[]}"#;

fn fresh_db() -> Arc<Database> {
    let conn = Connection::open_in_memory().expect("open in-memory");
    conn.execute_batch(SCHEMA_SQL).expect("apply schema");
    Arc::new(Database::from_connection(conn))
}

fn provider_for(server: &MockServer) -> Arc<MassiveProvider> {
    Arc::new(MassiveProvider::with_base_url(
        AppHttpClient::new(),
        Some("test_key".to_string()),
        MassiveTier::Paid,
        server.uri(),
    ))
}

fn noop_progress() -> Box<dyn Fn(SyncProgress) + Send + std::marker::Sync> {
    Box::new(|_| {})
}

/// Mount the four macro endpoints with empty results so a full run doesn't
/// 404 partway through. Callers that want to assert on macro behavior
/// should mount their own override before this is called (wiremock matches
/// in registration order).
async fn mount_empty_macros(server: &MockServer) {
    for p in [
        "/fed/v1/treasury-yields",
        "/fed/v1/inflation",
        "/fed/v1/labor-market",
    ] {
        Mock::given(method("GET"))
            .and(path(p))
            .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGED))
            .mount(server)
            .await;
    }
}

// ----------------------------------------------------------------------------

#[tokio::test]
async fn run_full_populates_db_against_mocked_massive() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/SPY"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(TICKER_OVERVIEW_AAPL.replace("AAPL", "SPY")),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/v2/aggs/ticker/SPY/range/1/day/.*$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(AGGS_SPY))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/stocks/v1/dividends"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGED))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/stocks/v1/splits"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGED))
        .mount(&server)
        .await;

    mount_empty_macros(&server).await;

    let db = fresh_db();
    {
        let conn = db.conn.lock().unwrap();
        settings::set_setting(&conn, SETTING_TRACKED_UNIVERSE, &vec!["SPY".to_string()])
            .expect("seed universe");
    }

    let sync = Sync::new(provider_for(&server), db.clone(), MassiveTier::Paid);
    let summary = sync
        .run(SyncMode::Full, noop_progress())
        .await
        .expect("full run should succeed");

    assert_eq!(summary.tickers_synced, 1);
    assert!(summary.errors.is_empty(), "errors: {:?}", summary.errors);
    assert!(summary.rows_upserted >= 8, "expected ≥8 SPY bars");

    let conn = db.conn.lock().unwrap();
    let bars = prices::get_prices(
        &conn,
        "SPY",
        NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
    )
    .unwrap();
    assert_eq!(bars.len(), 8);

    // sync_log row should be `completed`.
    let last_ok = sync_log::last_successful_sync(&conn).unwrap();
    assert!(last_ok.is_some(), "expected a completed sync_log row");
}

#[tokio::test]
async fn run_incremental_skips_when_prices_already_caught_up() {
    let server = MockServer::start().await;

    let aggs_hits = Arc::new(AtomicU32::new(0));
    let aggs_hits_for_mock = aggs_hits.clone();

    Mock::given(method("GET"))
        .and(path_regex(r"^/v2/aggs/ticker/CACHED/range/1/day/.*$"))
        .respond_with(move |_req: &Request| {
            aggs_hits_for_mock.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).set_body_string(EMPTY_AGGS)
        })
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/stocks/v1/dividends"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGED))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/stocks/v1/splits"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGED))
        .mount(&server)
        .await;

    mount_empty_macros(&server).await;

    let db = fresh_db();
    {
        let conn = db.conn.lock().unwrap();

        // Pre-populate an asset row + a price row dated tomorrow so
        // `compute_start_date` returns a start strictly past `today()`.
        conn.execute(
            "INSERT INTO assets (ticker, name, asset_type, source) VALUES (?, ?, ?, ?)",
            duckdb::params!["CACHED", "Cached Inc.", "stock", "massive"],
        )
        .unwrap();
        let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
        conn.execute(
            "INSERT INTO daily_prices (ticker, date, open, high, low, close, adj_close, volume, source)
             VALUES (?, ?, 1, 1, 1, 1, 1, 1, 'massive')",
            duckdb::params!["CACHED", tomorrow.to_string()],
        )
        .unwrap();

        settings::set_setting(&conn, SETTING_TRACKED_UNIVERSE, &vec!["CACHED".to_string()])
            .unwrap();
    }

    let sync = Sync::new(provider_for(&server), db.clone(), MassiveTier::Paid);
    let summary = sync
        .run(SyncMode::Incremental, noop_progress())
        .await
        .expect("incremental should succeed");

    assert_eq!(summary.tickers_synced, 1);
    assert_eq!(
        aggs_hits.load(Ordering::SeqCst),
        0,
        "incremental must not refetch when latest_price_date is already past today()"
    );
}

#[tokio::test]
async fn ensure_ticker_dedupes_concurrent_calls() {
    let server = MockServer::start().await;

    let prices_hits = Arc::new(AtomicU32::new(0));
    let prices_hits_for_mock = prices_hits.clone();

    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/SPY"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(TICKER_OVERVIEW_AAPL.replace("AAPL", "SPY")),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex(r"^/v2/aggs/ticker/SPY/range/1/day/.*$"))
        .respond_with(move |_req: &Request| {
            prices_hits_for_mock.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).set_body_string(AGGS_SPY)
        })
        .mount(&server)
        .await;

    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    let start = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
    let end = NaiveDate::from_ymd_opt(2025, 4, 10).unwrap();

    let s1 = sync.clone();
    let s2 = sync.clone();
    let (r1, r2) = tokio::join!(
        tokio::spawn(async move { s1.ensure_ticker("SPY", start, end).await }),
        tokio::spawn(async move { s2.ensure_ticker("SPY", start, end).await }),
    );
    r1.unwrap().expect("first ensure");
    r2.unwrap().expect("second ensure");

    assert_eq!(
        prices_hits.load(Ordering::SeqCst),
        1,
        "second call should observe stored prices and skip the network fetch"
    );
}

#[tokio::test]
async fn run_isolates_per_ticker_errors_and_continues() {
    let server = MockServer::start().await;

    // GOOD ticker: full happy path.
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/GOOD"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(TICKER_OVERVIEW_AAPL.replace("AAPL", "GOOD")),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/v2/aggs/ticker/GOOD/range/1/day/.*$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(AGGS_SPY))
        .mount(&server)
        .await;

    // BAD ticker: 404 on the metadata lookup.
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/BAD"))
        .respond_with(ResponseTemplate::new(404).set_body_string(TICKER_OVERVIEW_404))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/stocks/v1/dividends"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGED))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/stocks/v1/splits"))
        .respond_with(ResponseTemplate::new(200).set_body_string(EMPTY_PAGED))
        .mount(&server)
        .await;

    mount_empty_macros(&server).await;

    let db = fresh_db();
    {
        let conn = db.conn.lock().unwrap();
        settings::set_setting(
            &conn,
            SETTING_TRACKED_UNIVERSE,
            &vec!["BAD".to_string(), "GOOD".to_string()],
        )
        .unwrap();
    }

    let sync = Sync::new(provider_for(&server), db.clone(), MassiveTier::Paid);
    let summary = sync
        .run(SyncMode::Full, noop_progress())
        .await
        .expect("orchestrator should not fail on per-ticker errors");

    assert_eq!(summary.tickers_synced, 1, "GOOD only");
    assert_eq!(summary.errors.len(), 1);
    assert_eq!(summary.errors[0].ticker, "BAD");
    assert!(
        summary.errors[0].error.to_lowercase().contains("not found"),
        "expected 404 surfaced as not-found, got: {}",
        summary.errors[0].error
    );
}

#[tokio::test]
async fn ensure_ticker_retries_once_on_429_then_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(TICKER_OVERVIEW_AAPL))
        .mount(&server)
        .await;

    let attempts = Arc::new(AtomicU32::new(0));
    let attempts_for_mock = attempts.clone();
    Mock::given(method("GET"))
        .and(path_regex(r"^/v2/aggs/ticker/AAPL/range/1/day/.*$"))
        .respond_with(move |_req: &Request| {
            let n = attempts_for_mock.fetch_add(1, Ordering::SeqCst);
            if n == 0 {
                // First call: 429 with Retry-After: 0 so the retry runs
                // immediately and the test stays fast.
                ResponseTemplate::new(429)
                    .insert_header("retry-after", "0")
                    .set_body_string(r#"{"status":"ERROR","error":"slow down"}"#)
            } else {
                ResponseTemplate::new(200).set_body_string(AGGS_SPY)
            }
        })
        .mount(&server)
        .await;

    let db = fresh_db();
    let sync = Sync::new(provider_for(&server), db.clone(), MassiveTier::Paid);
    sync.ensure_ticker(
        "AAPL",
        NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
    )
    .await
    .expect("retry should recover");

    assert_eq!(attempts.load(Ordering::SeqCst), 2, "exactly one retry");
}
