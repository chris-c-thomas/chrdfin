use chrono::NaiveDate;
use duckdb::{Connection, params};

use crate::error::AppResult;
use crate::sync::types::DailyPrice;

use super::source::priority_case_sql;

/// Bulk insert/update of daily bars. Wrapped in a transaction so a partial
/// network response never leaves the table half-written. Source priority
/// guards the update branch.
pub fn upsert_prices(conn: &Connection, rows: &[DailyPrice], source: &str) -> AppResult<usize> {
    if rows.is_empty() {
        return Ok(0);
    }

    let sql = format!(
        r#"
        INSERT INTO daily_prices (
            ticker, date, open, high, low, close, adj_close, volume, source
        )
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT (ticker, date) DO UPDATE SET
            open      = EXCLUDED.open,
            high      = EXCLUDED.high,
            low       = EXCLUDED.low,
            close     = EXCLUDED.close,
            adj_close = EXCLUDED.adj_close,
            volume    = EXCLUDED.volume,
            source    = EXCLUDED.source
        WHERE {new} >= {old}
        "#,
        new = priority_case_sql("EXCLUDED.source"),
        old = priority_case_sql("daily_prices.source"),
    );

    conn.execute_batch("BEGIN TRANSACTION")?;
    let mut total = 0usize;
    {
        let mut stmt = conn.prepare(&sql)?;
        for r in rows {
            total += stmt.execute(params![
                r.ticker,
                r.date,
                r.open,
                r.high,
                r.low,
                r.close,
                r.adj_close,
                r.volume,
                source,
            ])?;
        }
    }
    conn.execute_batch("COMMIT")?;
    Ok(total)
}

/// Inclusive-range read.
pub fn get_prices(
    conn: &Connection,
    ticker: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> AppResult<Vec<DailyPrice>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ticker, date, open, high, low, close, adj_close, volume
        FROM daily_prices
        WHERE ticker = ? AND date BETWEEN ? AND ?
        ORDER BY date ASC
        "#,
    )?;
    let rows = stmt.query_map(params![ticker, start, end], |row| {
        Ok(DailyPrice {
            ticker: row.get(0)?,
            date: row.get(1)?,
            open: row.get(2)?,
            high: row.get(3)?,
            low: row.get(4)?,
            close: row.get(5)?,
            adj_close: row.get(6)?,
            volume: row.get(7)?,
        })
    })?;
    rows.collect::<duckdb::Result<Vec<_>>>().map_err(Into::into)
}

/// Latest stored date for a ticker — used by the orchestrator to compute
/// the start of an incremental fetch window.
pub fn latest_price_date(conn: &Connection, ticker: &str) -> AppResult<Option<NaiveDate>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT max(date) FROM daily_prices WHERE ticker = ?
        "#,
    )?;
    let mut rows = stmt.query(params![ticker])?;
    if let Some(row) = rows.next()? {
        Ok(row.get::<_, Option<NaiveDate>>(0)?)
    } else {
        Ok(None)
    }
}
