//! Coordinates provider fetches, storage upserts, and progress reporting.
//!
//! The orchestrator is the single entry point that knows what to fetch
//! when. Tauri command handlers (`commands::sync`) and the background
//! scheduler (sub-phase 1F) both go through `Sync::run`.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex as TokioMutex;

use crate::db::Database;
use crate::error::{AppError, AppResult};
use crate::secrets::MassiveTier;
use crate::storage::{assets, dividends, macros, prices, settings, splits, sync_log};
use crate::sync::error::ProviderError;
use crate::sync::massive::client::MassiveProvider;
use crate::sync::massive::limits::FREE_TIER_HISTORY_YEARS;
use crate::sync::provider::{DataProvider, MacroProvider};
use crate::sync::types::MacroSeriesId;

/// Settings key holding the JSON array of tickers to sync.
pub const SETTING_TRACKED_UNIVERSE: &str = "tracked_universe";

/// Macro series fetched on every full or incremental run. Phase 1F seed
/// also populates these into `app_settings.tracked_universe` for the
/// equity universe; the macro list is fixed in code.
pub const DEFAULT_MACRO_SERIES: &[MacroSeriesId] = &[
    MacroSeriesId::Treasury3Mo,
    MacroSeriesId::Treasury10Y,
    MacroSeriesId::CpiYoy,
    MacroSeriesId::UnemploymentRate,
];

/// Provider source string written to the `source` column on every row
/// produced by this run. Aligns with `storage::source::SOURCE_PRIORITY`.
const SOURCE: &str = "massive";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    /// Fetch from the asset's first date forward (clamped to ~2 years on
    /// the free tier, since older bars require a paid plan).
    Full,
    /// Fetch only rows newer than the latest stored date for each ticker.
    Incremental,
}

impl SyncMode {
    fn as_log_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Incremental => "incremental",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncProgress {
    pub phase: String,
    pub current: u32,
    pub total: u32,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncError {
    pub ticker: String,
    pub error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSummary {
    pub mode: SyncMode,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub tickers_synced: u32,
    pub rows_upserted: u32,
    pub errors: Vec<SyncError>,
}

/// Type-erased progress callback. The Tauri command (sub-phase 1D) passes
/// a closure that emits `sync:progress` events; tests pass a closure that
/// pushes into a Vec for assertions.
///
/// Fully-qualified `std::marker::Sync` to avoid colliding with the local
/// `Sync` orchestrator struct defined below.
pub type ProgressFn = Box<dyn Fn(SyncProgress) + Send + std::marker::Sync>;

pub struct Sync {
    massive: Arc<MassiveProvider>,
    db: Arc<Database>,
    tier: MassiveTier,
    /// Run-level mutex — `try_lock` returns busy if a sync (manual or
    /// scheduled) is already in flight, which prevents overlap.
    run_mutex: Arc<TokioMutex<()>>,
    /// Per-ticker mutex map for `ensure_ticker` dedup. Concurrent calls
    /// for the same ticker queue behind one another instead of issuing
    /// duplicate fetches.
    in_flight: DashMap<String, Arc<TokioMutex<()>>>,
}

impl Sync {
    pub fn new(massive: Arc<MassiveProvider>, db: Arc<Database>, tier: MassiveTier) -> Self {
        Self {
            massive,
            db,
            tier,
            run_mutex: Arc::new(TokioMutex::new(())),
            in_flight: DashMap::new(),
        }
    }

    /// Drive a full or incremental sync of the tracked universe + macro
    /// series. Per-ticker errors are isolated; one bad symbol never aborts
    /// the run. The whole invocation logs one `sync_log` row.
    pub async fn run(&self, mode: SyncMode, on_progress: ProgressFn) -> AppResult<SyncSummary> {
        let _run_lock = self
            .run_mutex
            .try_lock()
            .map_err(|_| AppError::InvalidInput("sync already running".to_string()))?;

        let started_at = Utc::now();
        let log_id = self.with_conn(|c| sync_log::start_sync(c, mode.as_log_str()))?;

        let outcome = self.run_inner(mode, &on_progress).await;

        let summary = match outcome {
            Ok((tickers_synced, rows_upserted, errors)) => {
                self.with_conn(|c| {
                    sync_log::complete_sync(c, &log_id, tickers_synced, rows_upserted)
                })?;
                SyncSummary {
                    mode,
                    started_at,
                    completed_at: Utc::now(),
                    tickers_synced,
                    rows_upserted,
                    errors,
                }
            }
            Err(e) => {
                let _ = self.with_conn(|c| sync_log::fail_sync(c, &log_id, &e.to_string()));
                return Err(e);
            }
        };

        Ok(summary)
    }

    /// On-demand fetch entry point. Used by the read commands (sub-phase
    /// 1E) when `get_prices` for an unknown ticker arrives.
    ///
    /// Per-ticker mutex guarantees concurrent callers for the same symbol
    /// dedupe to a single backend fetch.
    pub async fn ensure_ticker(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> AppResult<()> {
        let lock = self.ticker_lock(ticker);
        let _guard = lock.lock().await;

        // Fetch metadata first if the asset is unknown.
        let asset = self.with_conn(|c| assets::get_asset(c, ticker))?;
        if asset.is_none() {
            self.fetch_and_store_metadata(ticker).await?;
        }

        // Decide what gap (if any) to fill. If existing coverage already
        // overlaps `[start, end]` we still re-fetch the requested window —
        // simple, safe, and on the free tier the rate limiter naturally
        // dampens any over-fetching. Optimization opportunity for later.
        let latest = self.with_conn(|c| prices::latest_price_date(c, ticker))?;
        let fetch_start = match latest {
            Some(d) if d >= end => return Ok(()),
            Some(d) if d >= start => d.succ_opt().unwrap_or(start),
            _ => start,
        };

        self.fetch_and_store_prices_window(ticker, fetch_start, end)
            .await?;
        Ok(())
    }

    // ------------------------------------------------------------------
    // Internals
    // ------------------------------------------------------------------

    async fn run_inner(
        &self,
        mode: SyncMode,
        on_progress: &ProgressFn,
    ) -> AppResult<(u32, u32, Vec<SyncError>)> {
        let universe: Vec<String> = self
            .with_conn(|c| settings::get_setting(c, SETTING_TRACKED_UNIVERSE))?
            .unwrap_or_default();
        let total_tickers = universe.len() as u32;
        let total_macro = DEFAULT_MACRO_SERIES.len() as u32;

        let mut tickers_synced = 0u32;
        let mut rows_upserted = 0u32;
        let mut errors: Vec<SyncError> = Vec::new();

        for (idx, ticker) in universe.iter().enumerate() {
            on_progress(SyncProgress {
                phase: "prices".to_string(),
                current: idx as u32,
                total: total_tickers,
                message: Some(ticker.clone()),
            });

            match self.sync_ticker(ticker, mode).await {
                Ok(rows) => {
                    rows_upserted = rows_upserted.saturating_add(rows);
                    tickers_synced += 1;
                }
                Err(e) => {
                    tracing::warn!(%ticker, error = %e, "ticker sync failed; continuing");
                    errors.push(SyncError {
                        ticker: ticker.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        for (idx, series) in DEFAULT_MACRO_SERIES.iter().enumerate() {
            on_progress(SyncProgress {
                phase: "macro".to_string(),
                current: idx as u32,
                total: total_macro,
                message: Some(series.as_db_str()),
            });

            match self.sync_macro_series(*series, mode).await {
                Ok(rows) => {
                    rows_upserted = rows_upserted.saturating_add(rows);
                }
                Err(e) => {
                    tracing::warn!(?series, error = %e, "macro sync failed; continuing");
                    errors.push(SyncError {
                        ticker: series.as_db_str(),
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok((tickers_synced, rows_upserted, errors))
    }

    async fn sync_ticker(&self, ticker: &str, mode: SyncMode) -> AppResult<u32> {
        // Make sure the asset row exists — splits/dividends/prices all
        // reference assets(ticker) via FK.
        let existing = self.with_conn(|c| assets::get_asset(c, ticker))?;
        if existing.is_none() {
            self.fetch_and_store_metadata(ticker).await?;
        }

        let end = today();
        let start = self.compute_start_date(ticker, mode, existing.is_some())?;
        if start > end {
            return Ok(0);
        }

        let mut total = 0u32;

        // Prices
        let bars = retry_on_429(|| self.massive.fetch_prices(ticker, start, end)).await?;
        let n = self.with_conn(|c| prices::upsert_prices(c, &bars, SOURCE))?;
        total = total.saturating_add(n as u32);

        // Dividends
        let divs = retry_on_429(|| self.massive.fetch_dividends(ticker, start, end)).await?;
        let n = self.with_conn(|c| dividends::upsert_dividends(c, &divs, SOURCE))?;
        total = total.saturating_add(n as u32);

        // Splits
        let sp = retry_on_429(|| self.massive.fetch_splits(ticker, start, end)).await?;
        let n = self.with_conn(|c| splits::upsert_splits(c, &sp, SOURCE))?;
        total = total.saturating_add(n as u32);

        Ok(total)
    }

    async fn sync_macro_series(&self, series: MacroSeriesId, mode: SyncMode) -> AppResult<u32> {
        let end = today();
        let start = match mode {
            SyncMode::Incremental => {
                match self.with_conn(|c| macros::latest_macro_date(c, series))? {
                    Some(d) => d.succ_opt().unwrap_or(d),
                    None => self.history_clamp_start(),
                }
            }
            SyncMode::Full => self.history_clamp_start(),
        };
        if start > end {
            return Ok(0);
        }

        let obs = retry_on_429(|| self.massive.fetch_series(series, start, end)).await?;
        let n = self.with_conn(|c| macros::upsert_macro_observations(c, &obs, SOURCE))?;
        Ok(n as u32)
    }

    async fn fetch_and_store_metadata(&self, ticker: &str) -> AppResult<()> {
        let metadata = retry_on_429(|| self.massive.fetch_metadata(ticker)).await?;
        self.with_conn(|c| assets::upsert_asset(c, &metadata, SOURCE))?;
        Ok(())
    }

    async fn fetch_and_store_prices_window(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> AppResult<()> {
        let bars = retry_on_429(|| self.massive.fetch_prices(ticker, start, end)).await?;
        self.with_conn(|c| prices::upsert_prices(c, &bars, SOURCE))?;
        Ok(())
    }

    fn compute_start_date(
        &self,
        ticker: &str,
        mode: SyncMode,
        asset_exists: bool,
    ) -> AppResult<NaiveDate> {
        match mode {
            SyncMode::Incremental if asset_exists => {
                let latest = self.with_conn(|c| prices::latest_price_date(c, ticker))?;
                Ok(match latest {
                    Some(d) => d.succ_opt().unwrap_or(d),
                    None => self.history_clamp_start(),
                })
            }
            _ => Ok(self.history_clamp_start()),
        }
    }

    fn history_clamp_start(&self) -> NaiveDate {
        match self.tier {
            MassiveTier::Free => today() - chrono::Duration::days(365 * FREE_TIER_HISTORY_YEARS),
            MassiveTier::Paid => NaiveDate::from_ymd_opt(1990, 1, 1).expect("static date"),
        }
    }

    fn ticker_lock(&self, ticker: &str) -> Arc<TokioMutex<()>> {
        self.in_flight
            .entry(ticker.to_string())
            .or_insert_with(|| Arc::new(TokioMutex::new(())))
            .clone()
    }

    /// Run a closure with the DB mutex held. Encapsulates the lock-poison
    /// boilerplate so the call sites stay tight.
    fn with_conn<T, F>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&duckdb::Connection) -> AppResult<T>,
    {
        let conn = self
            .db
            .conn
            .lock()
            .map_err(|e| AppError::InvalidInput(format!("db mutex poisoned: {e}")))?;
        f(&conn)
    }
}

/// One transparent retry on a 429 — honors `Retry-After` if the provider
/// supplied it, otherwise falls back to a 12s wait (the free-tier
/// replenishment interval). A second 429 propagates to the caller, which
/// for `Sync::run` ends up in `SyncSummary.errors` rather than aborting.
async fn retry_on_429<T, Fut, F>(mut f: F) -> Result<T, ProviderError>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, ProviderError>>,
{
    match f().await {
        Ok(v) => Ok(v),
        Err(ProviderError::RateLimited { retry_after, .. }) => {
            let wait = retry_after.unwrap_or_else(|| Duration::from_secs(12));
            tracing::info!(?wait, "429 — sleeping before retry");
            tokio::time::sleep(wait).await;
            f().await
        }
        Err(e) => Err(e),
    }
}

fn today() -> NaiveDate {
    Utc::now().date_naive()
}
