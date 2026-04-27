use chrono::NaiveDate;
use duckdb::{Connection, params};

use crate::error::AppResult;
use crate::sync::types::{AssetMetadata, TickerSearchHit};

use super::source::priority_case_sql;

/// Insert or update one row in `assets`. Source priority guards the
/// update branch — a higher-priority source's existing row is left alone.
pub fn upsert_asset(conn: &Connection, asset: &AssetMetadata, source: &str) -> AppResult<usize> {
    let sql = format!(
        r#"
        INSERT INTO assets (
            ticker, name, asset_type, sector, industry, exchange,
            market_cap, first_date, last_date, is_active, source
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (ticker) DO UPDATE SET
            name        = EXCLUDED.name,
            asset_type  = EXCLUDED.asset_type,
            sector      = EXCLUDED.sector,
            industry    = EXCLUDED.industry,
            exchange    = EXCLUDED.exchange,
            market_cap  = EXCLUDED.market_cap,
            first_date  = EXCLUDED.first_date,
            last_date   = EXCLUDED.last_date,
            is_active   = EXCLUDED.is_active,
            source      = EXCLUDED.source,
            updated_at  = now()
        WHERE {new} >= {old}
        "#,
        new = priority_case_sql("EXCLUDED.source"),
        old = priority_case_sql("assets.source"),
    );

    let rows = conn.execute(
        &sql,
        params![
            asset.ticker,
            asset.name,
            asset.asset_type,
            asset.sector,
            asset.industry,
            asset.exchange,
            asset.market_cap,
            asset.first_date,
            asset.last_date,
            asset.is_active,
            source,
        ],
    )?;
    Ok(rows)
}

/// Fetch one asset by ticker. Returns `Ok(None)` if absent.
pub fn get_asset(conn: &Connection, ticker: &str) -> AppResult<Option<AssetMetadata>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ticker, name, asset_type, exchange, sector, industry,
               market_cap, first_date, last_date, is_active
        FROM assets
        WHERE ticker = ?
        "#,
    )?;
    let mut rows = stmt.query(params![ticker])?;
    if let Some(row) = rows.next()? {
        Ok(Some(AssetMetadata {
            ticker: row.get(0)?,
            name: row.get(1)?,
            asset_type: row.get(2)?,
            exchange: row.get(3)?,
            sector: row.get(4)?,
            industry: row.get(5)?,
            market_cap: row.get(6)?,
            first_date: row.get::<_, Option<NaiveDate>>(7)?,
            last_date: row.get::<_, Option<NaiveDate>>(8)?,
            is_active: row.get(9)?,
        }))
    } else {
        Ok(None)
    }
}

/// Local-first ticker search. Matches case-insensitively against ticker
/// or name. The orchestrator (1D) falls through to a live Massive search
/// when this returns fewer than `limit` hits.
pub fn search_assets_local(
    conn: &Connection,
    query: &str,
    limit: u32,
) -> AppResult<Vec<TickerSearchHit>> {
    let pattern = format!("%{}%", query.to_uppercase());
    let mut stmt = conn.prepare(
        r#"
        SELECT ticker, name, asset_type, exchange
        FROM assets
        WHERE is_active = true
          AND (UPPER(ticker) LIKE ? OR UPPER(name) LIKE ?)
        ORDER BY ticker ASC
        LIMIT ?
        "#,
    )?;
    let rows = stmt.query_map(params![pattern, pattern, limit as i64], |row| {
        Ok(TickerSearchHit {
            ticker: row.get(0)?,
            name: row.get(1)?,
            asset_type: row.get(2)?,
            exchange: row.get(3)?,
        })
    })?;
    rows.collect::<duckdb::Result<Vec<_>>>().map_err(Into::into)
}
