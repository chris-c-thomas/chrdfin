use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// One row of EOD OHLCV data for a single ticker on a single date.
///
/// Fields mirror what every adapter is expected to return after parsing
/// and normalization. Storage upserts (sub-phase 1C) consume this directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DailyPrice {
    pub ticker: String,
    pub date: NaiveDate,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub adj_close: f64,
    pub volume: i64,
}

/// One cash dividend distribution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dividend {
    pub ticker: String,
    pub ex_date: NaiveDate,
    pub amount: f64,
    pub div_type: DividendType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DividendType {
    Regular,
    Special,
    ReturnOfCapital,
    Unknown,
}

/// One stock split event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Split {
    pub ticker: String,
    pub execution_date: NaiveDate,
    /// Pre-split share count factor — e.g. `1.0` in a 4-for-1.
    pub split_from: f64,
    /// Post-split share count factor — e.g. `4.0` in a 4-for-1.
    pub split_to: f64,
    pub adjustment_type: SplitType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitType {
    ForwardSplit,
    ReverseSplit,
    StockDividend,
    Unknown,
}

/// A single ticker's reference + descriptive metadata. Provider responses
/// typically carry far more fields than this; adapters keep only the ones
/// the platform actually consumes and stash the rest in `extras`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetMetadata {
    pub ticker: String,
    pub name: String,
    pub asset_type: String,
    pub exchange: Option<String>,
    pub sector: Option<String>,
    pub industry: Option<String>,
    pub market_cap: Option<i64>,
    pub first_date: Option<NaiveDate>,
    pub last_date: Option<NaiveDate>,
    pub is_active: bool,
}

/// One row of an autocomplete / lookup response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TickerSearchHit {
    pub ticker: String,
    pub name: String,
    pub asset_type: Option<String>,
    pub exchange: Option<String>,
}

/// A single observation in a macroeconomic time series. Massive's `/fed/v1/*`
/// endpoints return per-date bundles (one row carrying every tenor of treasury
/// yield, every CPI flavor, etc.); adapters explode those bundles into one
/// `MacroObservation` per `series` so storage stays a clean
/// `(series_id, date) -> value` shape.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MacroObservation {
    pub series: MacroSeriesId,
    pub date: NaiveDate,
    pub value: f64,
}

/// Closed enum of macro series the platform tracks. Serialized form is the
/// `snake_case` variant name — that string is what lands in DuckDB's
/// `macro_series.series_id` column.
///
/// New series get added here as they're needed. Phase 1F seeds the four
/// `DEFAULT_MACRO_SERIES` (3-month treasury, 10-year treasury, CPI year-over-
/// year, unemployment rate); the rest are reserved for later phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroSeriesId {
    Treasury3Mo,
    Treasury2Y,
    Treasury10Y,
    Treasury30Y,
    CpiYoy,
    CpiCoreYoy,
    UnemploymentRate,
    FedFundsRate,
}
