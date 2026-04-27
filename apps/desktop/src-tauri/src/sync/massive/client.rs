//! `MassiveProvider` — the only adapter wired up in Phase 1.
//!
//! Holds the shared HTTP client, the bearer token, the per-provider rate
//! limiter, and the base URL (overridable for tests so wiremock can serve
//! fixtures locally). All outbound requests funnel through `get_json`,
//! which enforces the rate limit, attaches the bearer header, and maps
//! HTTP failures into the typed `ProviderError` taxonomy.

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::NaiveDate;
use reqwest::StatusCode;
use serde::de::DeserializeOwned;

use crate::http::AppHttpClient;
use crate::secrets::MassiveTier;
use crate::sync::error::{ProviderError, ProviderResult};
use crate::sync::provider::{DataProvider, MacroProvider};
use crate::sync::rate_limit::ProviderRateLimiter;
use crate::sync::types::{
    AssetMetadata, DailyPrice, Dividend, MacroObservation, MacroSeriesId, Split, TickerSearchHit,
};

use super::dto::{
    AggsResponse, DividendsResponse, ErrorEnvelope, InflationRow, LaborMarketRow,
    MacroBundleResponse, SplitsResponse, TickerOverviewResponse, TickersListResponse,
    TreasuryYieldsRow,
};
use super::mappers::{
    aggs_row_to_daily_price, dividend_row_to_dividend, inflation_to_observations,
    labor_to_observations, overview_to_metadata, split_row_to_split, ticker_list_row_to_search_hit,
    treasury_to_observations,
};

const PROVIDER_NAME: &str = "massive";
const DEFAULT_BASE_URL: &str = "https://api.massive.com";

/// How many `next_url` follows we'll do before bailing. Each follow costs
/// one rate-limit token; on the free tier (5 RPM) anything beyond a handful
/// is going to feel slow, and unbounded pagination invites runaway loops on
/// broad searches.
const MAX_PAGES: usize = 16;

#[derive(Clone)]
pub struct MassiveProvider {
    http: AppHttpClient,
    api_key: Option<String>,
    rate_limiter: Arc<ProviderRateLimiter>,
    base_url: String,
}

impl MassiveProvider {
    pub fn new(http: AppHttpClient, api_key: Option<String>, tier: MassiveTier) -> Self {
        Self {
            http,
            api_key,
            rate_limiter: Arc::new(ProviderRateLimiter::for_massive_tier(tier)),
            base_url: DEFAULT_BASE_URL.to_string(),
        }
    }

    /// Test-only constructor that points the adapter at a local mock server.
    pub fn with_base_url(
        http: AppHttpClient,
        api_key: Option<String>,
        tier: MassiveTier,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            http,
            api_key,
            rate_limiter: Arc::new(ProviderRateLimiter::for_massive_tier(tier)),
            base_url: base_url.into(),
        }
    }

    fn require_key(&self) -> ProviderResult<&str> {
        self.api_key
            .as_deref()
            .filter(|k| !k.is_empty())
            .ok_or(ProviderError::MissingApiKey(PROVIDER_NAME))
    }

    /// GET a relative path under `base_url` and decode JSON into `T`.
    async fn get_json<T: DeserializeOwned>(
        &self,
        path: &str,
        query: &[(&str, String)],
    ) -> ProviderResult<T> {
        let url = format!("{}{}", self.base_url, path);
        self.get_url::<T>(&url, query).await
    }

    /// GET an absolute URL (used for following `next_url`).
    async fn get_url<T: DeserializeOwned>(
        &self,
        url: &str,
        query: &[(&str, String)],
    ) -> ProviderResult<T> {
        let key = self.require_key()?;
        self.rate_limiter.until_ready().await;

        let response = self
            .http
            .inner()
            .get(url)
            .bearer_auth(key)
            .query(query)
            .send()
            .await?;

        let status = response.status();
        let request_url = response.url().to_string();

        if status.is_success() {
            // Decode the body with a serde_path_to_error wrapper so a bad
            // shape produces a useful error rather than an opaque "expected
            // string at line 1 column N".
            let bytes = response.bytes().await?;
            return serde_json::from_slice::<T>(&bytes).map_err(|e| {
                ProviderError::Decode(format!("decode failed for {request_url}: {e}"))
            });
        }

        let retry_after = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);

        let body = response.text().await.unwrap_or_default();
        let envelope = serde_json::from_str::<ErrorEnvelope>(&body).unwrap_or_default();

        Err(match status {
            StatusCode::NOT_FOUND => ProviderError::NotFound(envelope.detail()),
            StatusCode::TOO_MANY_REQUESTS => ProviderError::RateLimited {
                provider: PROVIDER_NAME,
                retry_after,
            },
            _ => ProviderError::BadStatus {
                status: status.as_u16(),
                url: request_url,
                body,
            },
        })
    }

    /// Walk `next_url` until exhausted (or `MAX_PAGES` hit), accumulating
    /// every row. `extract` pulls the rows + next link out of each page.
    async fn paginate<T, R, F>(
        &self,
        first_path: &str,
        first_query: &[(&str, String)],
        mut extract: F,
    ) -> ProviderResult<Vec<R>>
    where
        T: DeserializeOwned,
        F: FnMut(T) -> (Vec<R>, Option<String>),
    {
        let first: T = self.get_json(first_path, first_query).await?;
        let (mut acc, mut next) = extract(first);

        let mut pages = 1usize;
        while let Some(url) = next {
            if pages >= MAX_PAGES {
                tracing::warn!(%url, "massive pagination cap hit, truncating");
                break;
            }
            let page: T = self.get_url(&url, &[]).await?;
            let (rows, n) = extract(page);
            acc.extend(rows);
            next = n;
            pages += 1;
        }

        Ok(acc)
    }
}

// ---- DataProvider impl ------------------------------------------------------

#[async_trait]
impl DataProvider for MassiveProvider {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn search_tickers(
        &self,
        query: &str,
        limit: u32,
    ) -> ProviderResult<Vec<TickerSearchHit>> {
        let q = vec![
            ("search", query.to_string()),
            ("active", "true".to_string()),
            ("limit", limit.min(1000).to_string()),
        ];
        let response: TickersListResponse = self.get_json("/v3/reference/tickers", &q).await?;
        Ok(response
            .results
            .into_iter()
            .map(ticker_list_row_to_search_hit)
            .collect())
    }

    async fn fetch_metadata(&self, ticker: &str) -> ProviderResult<AssetMetadata> {
        let path = format!("/v3/reference/tickers/{ticker}");
        let response: TickerOverviewResponse = self.get_json(&path, &[]).await?;
        let overview = response
            .results
            .ok_or_else(|| ProviderError::NotFound(ticker.to_string()))?;
        overview_to_metadata(overview)
    }

    async fn fetch_prices(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<DailyPrice>> {
        let path = format!("/v2/aggs/ticker/{ticker}/range/1/day/{start}/{end}");
        let q = vec![
            ("adjusted", "true".to_string()),
            ("sort", "asc".to_string()),
            ("limit", "50000".to_string()),
        ];
        let response: AggsResponse = self.get_json(&path, &q).await?;
        let rows = response.results.unwrap_or_default();
        rows.iter()
            .map(|row| aggs_row_to_daily_price(ticker, row))
            .collect()
    }

    async fn fetch_dividends(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<Dividend>> {
        let q = vec![
            ("ticker", ticker.to_string()),
            ("ex_dividend_date.gte", start.to_string()),
            ("ex_dividend_date.lte", end.to_string()),
            ("limit", "1000".to_string()),
        ];
        let rows = self
            .paginate::<DividendsResponse, _, _>("/stocks/v1/dividends", &q, |page| {
                (page.results, page.next_url)
            })
            .await?;
        rows.into_iter().map(dividend_row_to_dividend).collect()
    }

    async fn fetch_splits(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<Split>> {
        let q = vec![
            ("ticker", ticker.to_string()),
            ("execution_date.gte", start.to_string()),
            ("execution_date.lte", end.to_string()),
            ("limit", "1000".to_string()),
        ];
        let rows = self
            .paginate::<SplitsResponse, _, _>("/stocks/v1/splits", &q, |page| {
                (page.results, page.next_url)
            })
            .await?;
        rows.into_iter().map(split_row_to_split).collect()
    }
}

// ---- MacroProvider impl -----------------------------------------------------

#[async_trait]
impl MacroProvider for MassiveProvider {
    fn name(&self) -> &'static str {
        PROVIDER_NAME
    }

    async fn fetch_series(
        &self,
        series: MacroSeriesId,
        start: NaiveDate,
        end: NaiveDate,
    ) -> ProviderResult<Vec<MacroObservation>> {
        match series {
            MacroSeriesId::Treasury3Mo | MacroSeriesId::Treasury10Y => {
                let q = date_range_query(start, end);
                let rows = self
                    .paginate::<MacroBundleResponse<TreasuryYieldsRow>, _, _>(
                        "/fed/v1/treasury-yields",
                        &q,
                        |page| (page.results, page.next_url),
                    )
                    .await?;
                treasury_to_observations(series, &rows)
            }
            MacroSeriesId::CpiYoy => {
                let q = date_range_query(start, end);
                let rows = self
                    .paginate::<MacroBundleResponse<InflationRow>, _, _>(
                        "/fed/v1/inflation",
                        &q,
                        |page| (page.results, page.next_url),
                    )
                    .await?;
                inflation_to_observations(series, &rows)
            }
            MacroSeriesId::UnemploymentRate => {
                let q = date_range_query(start, end);
                let rows = self
                    .paginate::<MacroBundleResponse<LaborMarketRow>, _, _>(
                        "/fed/v1/labor-market",
                        &q,
                        |page| (page.results, page.next_url),
                    )
                    .await?;
                labor_to_observations(series, &rows)
            }
        }
    }
}

fn date_range_query(start: NaiveDate, end: NaiveDate) -> Vec<(&'static str, String)> {
    vec![
        ("date.gte", start.to_string()),
        ("date.lte", end.to_string()),
        ("limit", "50000".to_string()),
    ]
}
