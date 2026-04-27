//! Raw response shapes for Massive endpoints.
//!
//! These mirror the JSON fields verbatim and exist only inside the adapter —
//! everything that crosses out of `crate::sync::massive` does so via the
//! mappers in `super::mappers`, which return the public DTOs from
//! `crate::sync::types`.

use serde::Deserialize;

// ---------- aggregates -------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AggsResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub ticker: Option<String>,
    #[serde(default)]
    pub results: Option<Vec<AggsRow>>,
}

#[derive(Debug, Deserialize)]
pub struct AggsRow {
    /// Epoch milliseconds at the start of the aggregate window (00:00 UTC
    /// for daily bars).
    pub t: i64,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    #[serde(default)]
    pub v: Option<f64>,
}

// ---------- ticker reference -------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct TickersListResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub results: Vec<TickerListRow>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TickerListRow {
    pub ticker: String,
    pub name: String,
    #[serde(default)]
    pub market: Option<String>,
    #[serde(default)]
    pub primary_exchange: Option<String>,
    #[serde(rename = "type", default)]
    pub asset_type: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct TickerOverviewResponse {
    pub status: String,
    #[serde(default)]
    pub results: Option<TickerOverview>,
}

#[derive(Debug, Deserialize)]
pub struct TickerOverview {
    pub ticker: String,
    pub name: String,
    #[serde(rename = "type", default)]
    pub asset_type: Option<String>,
    #[serde(default)]
    pub primary_exchange: Option<String>,
    /// Massive uses SIC industry classification; we surface the description
    /// as `industry` and leave `sector` empty until we add a SIC→sector map.
    #[serde(default)]
    pub sic_description: Option<String>,
    #[serde(default)]
    pub market_cap: Option<f64>,
    #[serde(default)]
    pub list_date: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
}

// ---------- corporate actions ------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct DividendsResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub results: Vec<DividendRow>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DividendRow {
    pub ticker: String,
    pub ex_dividend_date: String,
    pub cash_amount: f64,
    /// `recurring` | `special` | `supplemental` | `irregular` | `unknown`.
    /// Mappers downcast unknown variants to `DividendType::Unknown` so a
    /// future API addition never breaks decode.
    #[serde(default)]
    pub distribution_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SplitsResponse {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub results: Vec<SplitRow>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SplitRow {
    pub ticker: String,
    pub execution_date: String,
    pub split_from: f64,
    pub split_to: f64,
    /// `forward_split` | `reverse_split` | `stock_dividend`. Same unknown-
    /// variant safety net as `DividendRow::distribution_type`.
    #[serde(default)]
    pub adjustment_type: Option<String>,
}

// ---------- macro bundles ----------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct MacroBundleResponse<R> {
    #[serde(default)]
    pub status: Option<String>,
    // `default = "Vec::new"` instead of `#[serde(default)]` so the
    // generic R doesn't have to implement Default.
    #[serde(default = "Vec::new")]
    pub results: Vec<R>,
    #[serde(default)]
    pub next_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TreasuryYieldsRow {
    pub date: String,
    #[serde(default)]
    pub yield_3_month: Option<f64>,
    #[serde(default)]
    pub yield_10_year: Option<f64>,
    // Other tenors (1mo, 1y, 2y, 5y, 30y) are present in the API but not
    // currently mapped — add them here when a `MacroSeriesId` variant
    // lands that consumes them.
}

#[derive(Debug, Deserialize)]
pub struct InflationRow {
    pub date: String,
    /// Year-over-year CPI percent change.
    #[serde(default)]
    pub cpi_year_over_year: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct LaborMarketRow {
    pub date: String,
    #[serde(default)]
    pub unemployment_rate: Option<f64>,
}

// ---------- error envelopes --------------------------------------------------

/// Massive's error responses come in two shapes — `{ message }` for 404s,
/// `{ error }` for 401/429. Both are flattened into a single struct so the
/// adapter doesn't have to branch on which one is present.
#[derive(Debug, Default, Deserialize)]
pub struct ErrorEnvelope {
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

impl ErrorEnvelope {
    pub fn detail(&self) -> String {
        self.message
            .clone()
            .or_else(|| self.error.clone())
            .unwrap_or_else(|| "(no detail)".to_string())
    }
}
