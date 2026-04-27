//! Conversions from raw Massive response shapes (`super::dto`) into the
//! public DTOs in `crate::sync::types`. Pure functions — easy to unit-test
//! against a captured fixture without spinning up wiremock.

use chrono::{DateTime, NaiveDate, Utc};

use crate::sync::error::{ProviderError, ProviderResult};
use crate::sync::types::{
    AssetMetadata, DailyPrice, Dividend, DividendType, MacroObservation, MacroSeriesId, Split,
    SplitType, TickerSearchHit,
};

use super::dto::{
    AggsRow, DividendRow, InflationRow, LaborMarketRow, SplitRow, TickerListRow, TickerOverview,
    TreasuryYieldsRow,
};

/// Convert one aggregate bar. `t` is epoch milliseconds at start-of-day UTC;
/// the truncated date is what lands in DuckDB.
pub fn aggs_row_to_daily_price(ticker: &str, row: &AggsRow) -> ProviderResult<DailyPrice> {
    let date = DateTime::<Utc>::from_timestamp_millis(row.t)
        .ok_or_else(|| ProviderError::Decode(format!("invalid timestamp: {}", row.t)))?
        .date_naive();

    Ok(DailyPrice {
        ticker: ticker.to_string(),
        date,
        open: row.o,
        high: row.h,
        low: row.l,
        close: row.c,
        // Massive returns split-adjusted close when `adjusted=true` (the
        // adapter always passes that), so `adj_close == close` from this
        // endpoint. Dividend adjustment is applied separately by the
        // backtest engine using the dividends + splits tables.
        adj_close: row.c,
        volume: row.v.map(|v| v as i64).unwrap_or(0),
    })
}

pub fn ticker_list_row_to_search_hit(row: TickerListRow) -> TickerSearchHit {
    TickerSearchHit {
        ticker: row.ticker,
        name: row.name,
        asset_type: row.asset_type,
        exchange: row.primary_exchange,
    }
}

pub fn overview_to_metadata(overview: TickerOverview) -> ProviderResult<AssetMetadata> {
    let first_date = overview.list_date.as_deref().map(parse_date).transpose()?;
    Ok(AssetMetadata {
        ticker: overview.ticker,
        name: overview.name,
        asset_type: overview.asset_type.unwrap_or_else(|| "unknown".to_string()),
        exchange: overview.primary_exchange,
        // Massive doesn't expose a sector field on the overview — leave
        // empty until a SIC→sector mapping lands (Phase 7).
        sector: None,
        industry: overview.sic_description,
        market_cap: overview.market_cap.map(|m| m as i64),
        first_date,
        last_date: None,
        is_active: overview.active.unwrap_or(true),
    })
}

pub fn dividend_row_to_dividend(row: DividendRow) -> ProviderResult<Dividend> {
    Ok(Dividend {
        ticker: row.ticker,
        ex_date: parse_date(&row.ex_dividend_date)?,
        amount: row.cash_amount,
        div_type: classify_dividend(row.distribution_type.as_deref()),
    })
}

pub fn split_row_to_split(row: SplitRow) -> ProviderResult<Split> {
    Ok(Split {
        ticker: row.ticker,
        execution_date: parse_date(&row.execution_date)?,
        split_from: row.split_from,
        split_to: row.split_to,
        adjustment_type: classify_split(row.adjustment_type.as_deref()),
    })
}

/// Treasury bundles → per-series observations. Returns one obs for every
/// row whose mapped field is populated; rows with a missing value are
/// silently skipped (the API uses `null` for unobserved tenors).
pub fn treasury_to_observations(
    series: MacroSeriesId,
    rows: &[TreasuryYieldsRow],
) -> ProviderResult<Vec<MacroObservation>> {
    let extract: fn(&TreasuryYieldsRow) -> Option<f64> = match series {
        MacroSeriesId::Treasury3Mo => |r| r.yield_3_month,
        MacroSeriesId::Treasury10Y => |r| r.yield_10_year,
        _ => {
            return Err(ProviderError::Decode(format!(
                "treasury endpoint cannot satisfy {series:?}"
            )));
        }
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(value) = extract(row) {
            out.push(MacroObservation {
                series,
                date: parse_date(&row.date)?,
                value,
            });
        }
    }
    Ok(out)
}

pub fn inflation_to_observations(
    series: MacroSeriesId,
    rows: &[InflationRow],
) -> ProviderResult<Vec<MacroObservation>> {
    let extract: fn(&InflationRow) -> Option<f64> = match series {
        MacroSeriesId::CpiYoy => |r| r.cpi_year_over_year,
        _ => {
            return Err(ProviderError::Decode(format!(
                "inflation endpoint cannot satisfy {series:?}"
            )));
        }
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(value) = extract(row) {
            out.push(MacroObservation {
                series,
                date: parse_date(&row.date)?,
                value,
            });
        }
    }
    Ok(out)
}

pub fn labor_to_observations(
    series: MacroSeriesId,
    rows: &[LaborMarketRow],
) -> ProviderResult<Vec<MacroObservation>> {
    let extract: fn(&LaborMarketRow) -> Option<f64> = match series {
        MacroSeriesId::UnemploymentRate => |r| r.unemployment_rate,
        _ => {
            return Err(ProviderError::Decode(format!(
                "labor-market endpoint cannot satisfy {series:?}"
            )));
        }
    };

    let mut out = Vec::with_capacity(rows.len());
    for row in rows {
        if let Some(value) = extract(row) {
            out.push(MacroObservation {
                series,
                date: parse_date(&row.date)?,
                value,
            });
        }
    }
    Ok(out)
}

fn parse_date(s: &str) -> ProviderResult<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .map_err(|e| ProviderError::Decode(format!("bad date {s}: {e}")))
}

fn classify_dividend(raw: Option<&str>) -> DividendType {
    match raw.map(str::to_ascii_lowercase).as_deref() {
        Some("recurring") => DividendType::Regular,
        Some("special") | Some("supplemental") => DividendType::Special,
        // Massive doesn't currently emit `return_of_capital` on the cash-
        // dividend endpoint, but reserve the variant for the future.
        Some("return_of_capital") => DividendType::ReturnOfCapital,
        _ => DividendType::Unknown,
    }
}

fn classify_split(raw: Option<&str>) -> SplitType {
    match raw.map(str::to_ascii_lowercase).as_deref() {
        Some("forward_split") => SplitType::ForwardSplit,
        Some("reverse_split") => SplitType::ReverseSplit,
        Some("stock_dividend") => SplitType::StockDividend,
        _ => SplitType::Unknown,
    }
}
