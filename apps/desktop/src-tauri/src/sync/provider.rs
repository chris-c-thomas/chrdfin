use async_trait::async_trait;
use chrono::NaiveDate;

use super::error::ProviderResult;
use super::types::{
    AssetMetadata, DailyPrice, Dividend, MacroObservation, MacroSeriesId, Split, TickerSearchHit,
};

/// Equity-data provider abstraction. Every adapter (Massive in Phase 1, room
/// for additional sources later) implements this trait so call sites stay
/// provider-agnostic.
///
/// Methods take `&self` and `async fn` — adapters are expected to be cheaply
/// cloneable handles around an `Arc`'d `reqwest::Client` + rate limiter, so
/// concurrent calls are fine.
#[async_trait]
pub trait DataProvider: Send + Sync {
    /// Stable identifier for this provider. Persisted in DuckDB's `source`
    /// column on every row this adapter writes (sub-phase 1C). Lowercase,
    /// no spaces — e.g. `"massive"`.
    fn name(&self) -> &'static str;

    /// Free-text ticker / company-name search.
    async fn search_tickers(&self, query: &str, limit: u32)
    -> ProviderResult<Vec<TickerSearchHit>>;

    /// Single-ticker reference data.
    async fn fetch_metadata(&self, ticker: &str) -> ProviderResult<AssetMetadata>;

    /// Inclusive-range historical EOD bars.
    async fn fetch_prices(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<DailyPrice>>;

    /// Cash dividend events with ex-dates inside `[start, end]`.
    async fn fetch_dividends(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<Dividend>>;

    /// Stock split events with execution dates inside `[start, end]`.
    async fn fetch_splits(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<Split>>;
}

/// Macroeconomic-data provider abstraction. Kept separate from
/// `DataProvider` because providers and credentials don't always overlap —
/// even though Massive happens to serve both surfaces today.
#[async_trait]
pub trait MacroProvider: Send + Sync {
    fn name(&self) -> &'static str;

    /// Inclusive-range observations for a single series. Adapters that
    /// fetch a bundle endpoint (e.g. all treasury tenors at once) explode
    /// the response into per-series observations before returning.
    async fn fetch_series(
        &self,
        series: MacroSeriesId,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<MacroObservation>>;
}
