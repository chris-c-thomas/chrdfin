//! Integration tests for `commands::data` against mocked Massive + an
//! in-memory DuckDB. The tests target the `*_inner` entry points so they
//! don't need to stand up a Tauri runtime.

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use chrdfin_desktop_lib::{
    commands::data::{
        GetAssetMetadataInput, GetMacroSeriesInput, GetPricesInput, SearchTickersInput,
        get_asset_metadata_inner, get_macro_series_inner, get_prices_inner, search_tickers_inner,
    },
    db::Database,
    http::AppHttpClient,
    secrets::MassiveTier,
    storage::{assets, macros},
    sync::{
        massive::client::MassiveProvider,
        orchestrator::Sync,
        types::{AssetMetadata, MacroObservation, MacroSeriesId},
    },
};
use chrono::NaiveDate;
use duckdb::Connection;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, Request, ResponseTemplate};

const SCHEMA_SQL: &str = include_str!("../src/schema.sql");
const TICKER_OVERVIEW_AAPL: &str = include_str!("fixtures/massive/ticker_overview_aapl.json");
const AGGS_SPY: &str = include_str!("fixtures/massive/aggs_spy_2025-04-01_2025-04-10.json");
const TICKERS_SEARCH_APPLE: &str = include_str!("fixtures/massive/tickers_search_apple.json");
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

fn sample_asset(ticker: &str) -> AssetMetadata {
    AssetMetadata {
        ticker: ticker.to_string(),
        name: format!("{ticker} Inc."),
        asset_type: "stock".to_string(),
        exchange: Some("XNAS".to_string()),
        sector: None,
        industry: None,
        market_cap: None,
        first_date: None,
        last_date: None,
        is_active: true,
    }
}

// ---------------------------------------------------------------------------

#[tokio::test]
async fn get_prices_dispatches_on_demand_fetch_when_db_is_empty() {
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

    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    let bars = get_prices_inner(
        db,
        sync,
        GetPricesInput {
            ticker: "SPY".to_string(),
            start: NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
        },
    )
    .await
    .expect("fetch + read");

    assert_eq!(bars.len(), 8, "fixture has 8 SPY bars");
    assert_eq!(bars[0].ticker, "SPY");
}

#[tokio::test]
async fn get_prices_filters_inclusively_on_both_endpoints() {
    let server = MockServer::start().await;
    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    {
        let conn = db.conn.lock().unwrap();
        assets::upsert_asset(&conn, &sample_asset("XYZ"), "massive").unwrap();
        for d in 1..=5 {
            conn.execute(
                "INSERT INTO daily_prices (ticker, date, open, high, low, close, adj_close, volume, source)
                 VALUES (?, ?, 1, 1, 1, 1, 1, 1, 'massive')",
                duckdb::params!["XYZ", NaiveDate::from_ymd_opt(2025, 4, d).unwrap().to_string()],
            )
            .unwrap();
        }
    }

    // Pre-populate so ensure_ticker hits its early-return: latest >= end.
    // No mocks mounted ⇒ any HTTP call would fail.
    let bars = get_prices_inner(
        db,
        sync,
        GetPricesInput {
            ticker: "XYZ".to_string(),
            start: NaiveDate::from_ymd_opt(2025, 4, 2).unwrap(),
            end: NaiveDate::from_ymd_opt(2025, 4, 4).unwrap(),
        },
    )
    .await
    .expect("local read");

    assert_eq!(bars.len(), 3, "Apr 2, 3, 4 inclusive");
    assert_eq!(bars[0].date, NaiveDate::from_ymd_opt(2025, 4, 2).unwrap());
    assert_eq!(bars[2].date, NaiveDate::from_ymd_opt(2025, 4, 4).unwrap());
}

#[tokio::test]
async fn get_prices_rejects_inverted_range() {
    let server = MockServer::start().await;
    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    let err = get_prices_inner(
        db,
        sync,
        GetPricesInput {
            ticker: "X".to_string(),
            start: NaiveDate::from_ymd_opt(2025, 4, 10).unwrap(),
            end: NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
        },
    )
    .await
    .expect_err("inverted range should be rejected");
    assert!(err.to_string().contains("must be <="));
}

#[tokio::test]
async fn get_macro_series_returns_local_observations() {
    let server = MockServer::start().await;
    let db = fresh_db();

    {
        let conn = db.conn.lock().unwrap();
        let obs = vec![
            MacroObservation {
                series: MacroSeriesId::Treasury10Y,
                date: NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
                value: 4.30,
            },
            MacroObservation {
                series: MacroSeriesId::Treasury10Y,
                date: NaiveDate::from_ymd_opt(2025, 4, 2).unwrap(),
                value: 4.32,
            },
        ];
        macros::upsert_macro_observations(&conn, &obs, "massive").unwrap();
    }

    let _sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));
    let series = get_macro_series_inner(
        &db,
        GetMacroSeriesInput {
            series_id: MacroSeriesId::Treasury10Y,
            start: NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            end: NaiveDate::from_ymd_opt(2025, 4, 30).unwrap(),
        },
    )
    .expect("read");
    assert_eq!(series.len(), 2);
    assert!((series[0].value - 4.30).abs() < 1e-9);
}

#[tokio::test]
async fn get_asset_metadata_local_first_avoids_network() {
    let server = MockServer::start().await;
    // Mount nothing — any HTTP call would fail the test.
    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    {
        let conn = db.conn.lock().unwrap();
        assets::upsert_asset(&conn, &sample_asset("AAPL"), "massive").unwrap();
    }

    let asset = get_asset_metadata_inner(
        db,
        sync,
        GetAssetMetadataInput {
            ticker: "AAPL".to_string(),
        },
    )
    .await
    .expect("read");
    assert_eq!(asset.ticker, "AAPL");
    assert_eq!(asset.name, "AAPL Inc.");
}

#[tokio::test]
async fn get_asset_metadata_falls_through_to_ensure_ticker_when_missing() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(TICKER_OVERVIEW_AAPL))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path_regex(r"^/v2/aggs/ticker/AAPL/range/1/day/.*$"))
        .respond_with(ResponseTemplate::new(200).set_body_string(AGGS_SPY))
        .mount(&server)
        .await;

    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    let asset = get_asset_metadata_inner(
        db,
        sync,
        GetAssetMetadataInput {
            ticker: "AAPL".to_string(),
        },
    )
    .await
    .expect("on-demand fetch");
    assert_eq!(asset.ticker, "AAPL");
}

#[tokio::test]
async fn search_tickers_local_only_when_local_satisfies_limit() {
    let server = MockServer::start().await;
    let remote_calls = Arc::new(AtomicU32::new(0));
    let remote_calls_for_mock = remote_calls.clone();
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers"))
        .respond_with(move |_req: &Request| {
            remote_calls_for_mock.fetch_add(1, Ordering::SeqCst);
            ResponseTemplate::new(200).set_body_string(EMPTY_PAGED)
        })
        .mount(&server)
        .await;

    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    {
        let conn = db.conn.lock().unwrap();
        for t in ["APPLE1", "APPLE2"] {
            assets::upsert_asset(&conn, &sample_asset(t), "massive").unwrap();
        }
    }

    let resp = search_tickers_inner(
        db,
        sync,
        SearchTickersInput {
            query: "APPLE".to_string(),
            limit: Some(2),
        },
    )
    .await
    .unwrap();

    assert_eq!(resp.hits.len(), 2);
    assert_eq!(remote_calls.load(Ordering::SeqCst), 0, "remote skipped");
}

#[tokio::test]
async fn search_tickers_merges_remote_when_local_underflows() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers"))
        .respond_with(ResponseTemplate::new(200).set_body_string(TICKERS_SEARCH_APPLE))
        .mount(&server)
        .await;

    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    // Seed AAPL locally so the merge path has a known dedupe target.
    {
        let conn = db.conn.lock().unwrap();
        assets::upsert_asset(&conn, &sample_asset("AAPL"), "massive").unwrap();
    }

    let resp = search_tickers_inner(
        db,
        sync,
        SearchTickersInput {
            query: "apple".to_string(),
            limit: Some(5),
        },
    )
    .await
    .unwrap();

    assert!(resp.hits.len() <= 5);
    let aapl_count = resp.hits.iter().filter(|h| h.ticker == "AAPL").count();
    assert_eq!(aapl_count, 1, "local AAPL must dedupe remote AAPL");
    assert!(
        resp.hits.len() > 1,
        "remote rows should fill in alongside local"
    );
}

#[tokio::test]
async fn search_tickers_swallows_remote_failure_and_returns_local() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v3/reference/tickers"))
        .respond_with(ResponseTemplate::new(500).set_body_string(r#"{"status":"ERROR"}"#))
        .mount(&server)
        .await;

    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));
    {
        let conn = db.conn.lock().unwrap();
        assets::upsert_asset(&conn, &sample_asset("AAPL"), "massive").unwrap();
    }

    let resp = search_tickers_inner(
        db,
        sync,
        SearchTickersInput {
            query: "AAPL".to_string(),
            limit: Some(10),
        },
    )
    .await
    .expect("remote 500 should not propagate");

    assert_eq!(resp.hits.len(), 1);
    assert_eq!(resp.hits[0].ticker, "AAPL");
}

#[tokio::test]
async fn search_tickers_empty_query_returns_empty() {
    let server = MockServer::start().await;
    let db = fresh_db();
    let sync = Arc::new(Sync::new(
        provider_for(&server),
        db.clone(),
        MassiveTier::Paid,
    ));

    let resp = search_tickers_inner(
        db,
        sync,
        SearchTickersInput {
            query: "   ".to_string(),
            limit: Some(10),
        },
    )
    .await
    .unwrap();
    assert!(resp.hits.is_empty());
}
