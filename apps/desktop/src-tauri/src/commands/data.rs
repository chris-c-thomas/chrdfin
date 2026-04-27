//! Local-first read commands the frontend uses for prices, macro series,
//! asset metadata, and ticker search.
//!
//! "Local-first" = always look in DuckDB before going to the network. The
//! orchestrator's `Sync::ensure_ticker` is the on-demand escape hatch:
//! when a chart asks for a ticker that has never been synced, the price
//! and metadata commands transparently fetch + store + re-read.
//!
//! Search is hybrid: local hits first (instant), then a remote merge if
//! the user typed something the local universe doesn't cover yet.
//!
//! Bodies are factored into `*_inner` functions that take `&Database` +
//! `&Sync` directly so integration tests can exercise them without
//! standing up a Tauri app.

use std::sync::Arc;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::storage::{assets, macros, prices};
use crate::sync::orchestrator::Sync;
use crate::sync::types::{
    AssetMetadata, DailyPrice, MacroObservation, MacroSeriesId, TickerSearchHit,
};

const SEARCH_DEFAULT_LIMIT: u32 = 10;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPricesInput {
    pub ticker: String,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[tauri::command]
pub async fn get_prices(
    state: State<'_, AppState>,
    input: GetPricesInput,
) -> AppResult<Vec<DailyPrice>> {
    get_prices_inner(state.db.clone(), state.sync.clone(), input).await
}

pub async fn get_prices_inner(
    db: Arc<Database>,
    sync: Arc<Sync>,
    input: GetPricesInput,
) -> AppResult<Vec<DailyPrice>> {
    let GetPricesInput { ticker, start, end } = input;
    if start > end {
        return Err(AppError::InvalidInput(format!(
            "start ({start}) must be <= end ({end})"
        )));
    }
    sync.ensure_ticker(&ticker, start, end).await?;
    with_conn(&db, |c| prices::get_prices(c, &ticker, start, end))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetMacroSeriesInput {
    pub series_id: MacroSeriesId,
    pub start: NaiveDate,
    pub end: NaiveDate,
}

#[tauri::command]
pub fn get_macro_series(
    state: State<'_, AppState>,
    input: GetMacroSeriesInput,
) -> AppResult<Vec<MacroObservation>> {
    get_macro_series_inner(&state.db, input)
}

pub fn get_macro_series_inner(
    db: &Database,
    input: GetMacroSeriesInput,
) -> AppResult<Vec<MacroObservation>> {
    let GetMacroSeriesInput {
        series_id,
        start,
        end,
    } = input;
    if start > end {
        return Err(AppError::InvalidInput(format!(
            "start ({start}) must be <= end ({end})"
        )));
    }
    with_conn(db, |c| macros::get_macro_series(c, series_id, start, end))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetAssetMetadataInput {
    pub ticker: String,
}

#[tauri::command]
pub async fn get_asset_metadata(
    state: State<'_, AppState>,
    input: GetAssetMetadataInput,
) -> AppResult<AssetMetadata> {
    get_asset_metadata_inner(state.db.clone(), state.sync.clone(), input).await
}

pub async fn get_asset_metadata_inner(
    db: Arc<Database>,
    sync: Arc<Sync>,
    input: GetAssetMetadataInput,
) -> AppResult<AssetMetadata> {
    let ticker = input.ticker;

    if let Some(asset) = with_conn(&db, |c| assets::get_asset(c, &ticker))? {
        return Ok(asset);
    }

    // Use a single-day window so `ensure_ticker` always populates the
    // `assets` row first; the price call may end up empty, which is fine.
    let today = chrono::Utc::now().date_naive();
    sync.ensure_ticker(&ticker, today, today).await?;

    with_conn(&db, |c| assets::get_asset(c, &ticker))?.ok_or_else(|| AppError::NotFound(ticker))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTickersInput {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchTickersResponse {
    pub hits: Vec<TickerSearchHit>,
}

#[tauri::command]
pub async fn search_tickers(
    state: State<'_, AppState>,
    input: SearchTickersInput,
) -> AppResult<SearchTickersResponse> {
    search_tickers_inner(state.db.clone(), state.sync.clone(), input).await
}

pub async fn search_tickers_inner(
    db: Arc<Database>,
    sync: Arc<Sync>,
    input: SearchTickersInput,
) -> AppResult<SearchTickersResponse> {
    let SearchTickersInput { query, limit } = input;
    let q = query.trim();
    if q.is_empty() {
        return Ok(SearchTickersResponse { hits: Vec::new() });
    }
    let limit = limit.unwrap_or(SEARCH_DEFAULT_LIMIT).max(1);

    let local = with_conn(&db, |c| assets::search_assets_local(c, q, limit))?;
    if local.len() as u32 >= limit {
        return Ok(SearchTickersResponse { hits: local });
    }

    let remote = match sync.search_tickers_remote(q, limit).await {
        Ok(hits) => hits,
        Err(err) => {
            tracing::warn!(query = %q, %err, "remote ticker search failed; returning local hits");
            return Ok(SearchTickersResponse { hits: local });
        }
    };

    let mut merged = local;
    let known: std::collections::HashSet<String> =
        merged.iter().map(|h| h.ticker.clone()).collect();
    for hit in remote {
        if merged.len() as u32 >= limit {
            break;
        }
        if !known.contains(&hit.ticker) {
            merged.push(hit);
        }
    }

    Ok(SearchTickersResponse { hits: merged })
}

fn with_conn<T, F>(db: &Database, f: F) -> AppResult<T>
where
    F: FnOnce(&duckdb::Connection) -> AppResult<T>,
{
    let conn = db
        .conn
        .lock()
        .map_err(|e| AppError::InvalidInput(format!("db mutex poisoned: {e}")))?;
    f(&conn)
}
