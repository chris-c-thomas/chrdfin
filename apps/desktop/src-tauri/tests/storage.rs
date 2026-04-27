//! Integration tests for the `storage` module against an in-memory DuckDB.
//!
//! Each test bootstraps a fresh `:memory:` connection, applies the canonical
//! schema (including the additive Phase 1C migration), and exercises one
//! storage helper end-to-end. No real filesystem or network involved.

use chrdfin_desktop_lib::{
    storage::{assets, dividends, macros, prices, settings, source, splits, sync_log},
    sync::types::{
        AssetMetadata, DailyPrice, Dividend, DividendType, MacroObservation, MacroSeriesId, Split,
        SplitType,
    },
};
use chrono::NaiveDate;
use duckdb::Connection;

const SCHEMA_SQL: &str = include_str!("../src/schema.sql");

fn fresh_db() -> Connection {
    let conn = Connection::open_in_memory().expect("open in-memory");
    conn.execute_batch(SCHEMA_SQL).expect("apply schema");
    conn
}

fn sample_asset(ticker: &str) -> AssetMetadata {
    AssetMetadata {
        ticker: ticker.to_string(),
        name: format!("{ticker} Inc."),
        asset_type: "stock".to_string(),
        exchange: Some("XNAS".to_string()),
        sector: None,
        industry: Some("Tech".to_string()),
        market_cap: Some(1_000_000_000),
        first_date: Some(NaiveDate::from_ymd_opt(1990, 1, 1).unwrap()),
        last_date: None,
        is_active: true,
    }
}

fn sample_price(ticker: &str, date: NaiveDate, close: f64) -> DailyPrice {
    DailyPrice {
        ticker: ticker.to_string(),
        date,
        open: close - 1.0,
        high: close + 1.0,
        low: close - 2.0,
        close,
        adj_close: close,
        volume: 1_000_000,
    }
}

// ---------- assets -----------------------------------------------------------

#[test]
fn upsert_asset_inserts_then_updates_same_source() {
    let conn = fresh_db();
    let mut a = sample_asset("AAPL");

    assets::upsert_asset(&conn, &a, "massive").expect("first insert");
    let stored = assets::get_asset(&conn, "AAPL")
        .expect("read")
        .expect("present");
    assert_eq!(stored.industry.as_deref(), Some("Tech"));

    a.industry = Some("Consumer Electronics".to_string());
    assets::upsert_asset(&conn, &a, "massive").expect("second upsert");
    let stored = assets::get_asset(&conn, "AAPL")
        .expect("read")
        .expect("present");
    assert_eq!(stored.industry.as_deref(), Some("Consumer Electronics"));
}

#[test]
fn search_assets_local_matches_ticker_or_name_case_insensitive() {
    let conn = fresh_db();
    assets::upsert_asset(&conn, &sample_asset("AAPL"), "massive").unwrap();
    assets::upsert_asset(&conn, &sample_asset("MSFT"), "massive").unwrap();

    let hits = assets::search_assets_local(&conn, "aapl", 5).unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].ticker, "AAPL");

    // Name-side match: "AAPL Inc." contains "Inc"
    let hits = assets::search_assets_local(&conn, "Inc", 5).unwrap();
    assert_eq!(hits.len(), 2);
}

// ---------- prices -----------------------------------------------------------

#[test]
fn upsert_prices_inserts_new_rows() {
    let conn = fresh_db();
    assets::upsert_asset(&conn, &sample_asset("SPY"), "massive").unwrap();

    let rows = vec![
        sample_price("SPY", NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(), 560.0),
        sample_price("SPY", NaiveDate::from_ymd_opt(2025, 4, 2).unwrap(), 565.0),
    ];
    let n = prices::upsert_prices(&conn, &rows, "massive").expect("upsert");
    assert_eq!(n, 2);

    let read = prices::get_prices(
        &conn,
        "SPY",
        NaiveDate::from_ymd_opt(2025, 3, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 4, 30).unwrap(),
    )
    .unwrap();
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].close, 560.0);
}

#[test]
fn upsert_prices_updates_same_source_on_conflict() {
    let conn = fresh_db();
    assets::upsert_asset(&conn, &sample_asset("SPY"), "massive").unwrap();
    let date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();

    prices::upsert_prices(&conn, &[sample_price("SPY", date, 560.0)], "massive").unwrap();
    prices::upsert_prices(&conn, &[sample_price("SPY", date, 999.0)], "massive").unwrap();

    let read = prices::get_prices(&conn, "SPY", date, date).unwrap();
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].close, 999.0, "same-source upsert should overwrite");
}

#[test]
fn upsert_prices_does_not_overwrite_higher_priority_source() {
    let conn = fresh_db();
    assets::upsert_asset(&conn, &sample_asset("SPY"), "massive").unwrap();
    let date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();

    // Pre-seed a row from the high-priority bulk-backfill source.
    prices::upsert_prices(
        &conn,
        &[sample_price("SPY", date, 560.0)],
        "manual_backfill",
    )
    .unwrap();

    // A massive (lower-priority) write must NOT clobber the backfill row.
    prices::upsert_prices(&conn, &[sample_price("SPY", date, 999.0)], "massive").unwrap();

    let read = prices::get_prices(&conn, "SPY", date, date).unwrap();
    assert_eq!(read[0].close, 560.0, "higher-priority row stays put");
}

#[test]
fn latest_price_date_returns_none_then_max() {
    let conn = fresh_db();
    assets::upsert_asset(&conn, &sample_asset("SPY"), "massive").unwrap();

    assert_eq!(prices::latest_price_date(&conn, "SPY").unwrap(), None);

    let dates = [
        NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 4, 3).unwrap(),
        NaiveDate::from_ymd_opt(2025, 4, 2).unwrap(),
    ];
    let rows: Vec<_> = dates
        .iter()
        .map(|d| sample_price("SPY", *d, 560.0))
        .collect();
    prices::upsert_prices(&conn, &rows, "massive").unwrap();

    assert_eq!(
        prices::latest_price_date(&conn, "SPY").unwrap(),
        Some(NaiveDate::from_ymd_opt(2025, 4, 3).unwrap())
    );
}

#[test]
fn upsert_prices_empty_input_is_noop() {
    let conn = fresh_db();
    let n = prices::upsert_prices(&conn, &[], "massive").unwrap();
    assert_eq!(n, 0);
}

// ---------- dividends + splits ----------------------------------------------

#[test]
fn upsert_and_read_dividends_round_trip() {
    let conn = fresh_db();
    assets::upsert_asset(&conn, &sample_asset("AAPL"), "massive").unwrap();

    let div = Dividend {
        ticker: "AAPL".to_string(),
        ex_date: NaiveDate::from_ymd_opt(2024, 11, 8).unwrap(),
        amount: 0.25,
        div_type: DividendType::Regular,
    };
    dividends::upsert_dividends(&conn, std::slice::from_ref(&div), "massive").unwrap();

    let read = dividends::get_dividends(
        &conn,
        "AAPL",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2024, 12, 31).unwrap(),
    )
    .unwrap();
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].amount, 0.25);
    assert!(matches!(read[0].div_type, DividendType::Regular));
}

#[test]
fn upsert_and_read_splits_round_trip() {
    let conn = fresh_db();
    assets::upsert_asset(&conn, &sample_asset("AAPL"), "massive").unwrap();

    let split = Split {
        ticker: "AAPL".to_string(),
        execution_date: NaiveDate::from_ymd_opt(2020, 8, 31).unwrap(),
        split_from: 1.0,
        split_to: 4.0,
        adjustment_type: SplitType::ForwardSplit,
    };
    splits::upsert_splits(&conn, &[split], "massive").unwrap();

    let read = splits::get_splits(&conn, "AAPL").unwrap();
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].split_to, 4.0);
    assert!(matches!(read[0].adjustment_type, SplitType::ForwardSplit));
}

// ---------- macros -----------------------------------------------------------

#[test]
fn upsert_and_read_macro_observations() {
    let conn = fresh_db();

    let obs = vec![
        MacroObservation {
            series: MacroSeriesId::Treasury10Y,
            date: NaiveDate::from_ymd_opt(2025, 4, 1).unwrap(),
            value: 4.17,
        },
        MacroObservation {
            series: MacroSeriesId::Treasury10Y,
            date: NaiveDate::from_ymd_opt(2025, 4, 2).unwrap(),
            value: 4.20,
        },
        MacroObservation {
            series: MacroSeriesId::CpiYoy,
            date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
            value: 3.0,
        },
    ];
    macros::upsert_macro_observations(&conn, &obs, "massive").unwrap();

    let read = macros::get_macro_series(
        &conn,
        MacroSeriesId::Treasury10Y,
        NaiveDate::from_ymd_opt(2025, 1, 1).unwrap(),
        NaiveDate::from_ymd_opt(2025, 12, 31).unwrap(),
    )
    .unwrap();
    assert_eq!(read.len(), 2, "treasury 10y observations");
    assert!(read.iter().all(|o| o.series == MacroSeriesId::Treasury10Y));

    assert_eq!(
        macros::latest_macro_date(&conn, MacroSeriesId::Treasury10Y).unwrap(),
        Some(NaiveDate::from_ymd_opt(2025, 4, 2).unwrap())
    );
    assert_eq!(
        macros::latest_macro_date(&conn, MacroSeriesId::UnemploymentRate).unwrap(),
        None,
    );
}

#[test]
fn macro_upsert_dedupes_on_series_and_date() {
    let conn = fresh_db();
    let date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
    let obs1 = MacroObservation {
        series: MacroSeriesId::Treasury10Y,
        date,
        value: 4.17,
    };
    let obs2 = MacroObservation {
        series: MacroSeriesId::Treasury10Y,
        date,
        value: 4.20, // corrected value
    };

    macros::upsert_macro_observations(&conn, &[obs1], "massive").unwrap();
    macros::upsert_macro_observations(&conn, &[obs2], "massive").unwrap();

    let read = macros::get_macro_series(&conn, MacroSeriesId::Treasury10Y, date, date).unwrap();
    assert_eq!(read.len(), 1, "(series, date) is the PK — no duplicates");
    assert_eq!(read[0].value, 4.20);
}

// ---------- sync_log ---------------------------------------------------------

#[test]
fn sync_log_start_complete_pair_persists_status() {
    let conn = fresh_db();
    let id = sync_log::start_sync(&conn, "incremental").unwrap();
    assert!(!id.is_empty(), "sync id should be a uuid string");

    sync_log::complete_sync(&conn, &id, 26, 1287).unwrap();

    let last = sync_log::last_successful_sync(&conn).unwrap();
    assert!(last.is_some(), "should have a successful sync timestamp");
}

#[test]
fn sync_log_failed_sync_does_not_count_as_successful() {
    let conn = fresh_db();
    let id = sync_log::start_sync(&conn, "full").unwrap();
    sync_log::fail_sync(&conn, &id, "rate limited").unwrap();

    assert_eq!(
        sync_log::last_successful_sync(&conn).unwrap(),
        None,
        "failed runs must not surface in last_successful_sync"
    );
}

// ---------- settings ---------------------------------------------------------

#[test]
fn app_settings_round_trip_a_json_value() {
    let conn = fresh_db();
    let universe: Vec<String> = vec!["SPY".into(), "QQQ".into(), "AGG".into()];

    settings::set_setting(&conn, "tracked_universe", &universe).unwrap();
    let read: Vec<String> = settings::get_setting(&conn, "tracked_universe")
        .unwrap()
        .expect("setting should exist");
    assert_eq!(read, universe);
}

#[test]
fn app_settings_returns_none_for_missing_key() {
    let conn = fresh_db();
    let read: Option<bool> = settings::get_setting(&conn, "nonexistent").unwrap();
    assert!(read.is_none());
}

// ---------- source priority --------------------------------------------------

#[test]
fn priority_helper_known_and_unknown_sources() {
    assert_eq!(source::priority("manual_backfill"), 100);
    assert_eq!(source::priority("massive"), 50);
    assert_eq!(source::priority("never_heard_of_it"), 0);
}
