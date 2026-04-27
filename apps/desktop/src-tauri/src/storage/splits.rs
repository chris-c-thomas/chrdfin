use duckdb::{Connection, params};

use crate::error::AppResult;
use crate::sync::types::{Split, SplitType};

use super::source::priority_case_sql;

pub fn upsert_splits(conn: &Connection, rows: &[Split], source: &str) -> AppResult<usize> {
    if rows.is_empty() {
        return Ok(0);
    }

    let sql = format!(
        r#"
        INSERT INTO splits (
            ticker, execution_date, split_from, split_to, adjustment_type, source
        )
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT (ticker, execution_date) DO UPDATE SET
            split_from      = EXCLUDED.split_from,
            split_to        = EXCLUDED.split_to,
            adjustment_type = EXCLUDED.adjustment_type,
            source          = EXCLUDED.source
        WHERE {new} >= {old}
        "#,
        new = priority_case_sql("EXCLUDED.source"),
        old = priority_case_sql("splits.source"),
    );

    conn.execute_batch("BEGIN TRANSACTION")?;
    let mut total = 0usize;
    {
        let mut stmt = conn.prepare(&sql)?;
        for s in rows {
            total += stmt.execute(params![
                s.ticker,
                s.execution_date,
                s.split_from,
                s.split_to,
                split_type_to_str(s.adjustment_type),
                source,
            ])?;
        }
    }
    conn.execute_batch("COMMIT")?;
    Ok(total)
}

pub fn get_splits(conn: &Connection, ticker: &str) -> AppResult<Vec<Split>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ticker, execution_date, split_from, split_to, adjustment_type
        FROM splits
        WHERE ticker = ?
        ORDER BY execution_date DESC
        "#,
    )?;
    let rows = stmt.query_map(params![ticker], |row| {
        let kind_str: Option<String> = row.get(4)?;
        Ok(Split {
            ticker: row.get(0)?,
            execution_date: row.get(1)?,
            split_from: row.get(2)?,
            split_to: row.get(3)?,
            adjustment_type: str_to_split_type(kind_str.as_deref()),
        })
    })?;
    rows.collect::<duckdb::Result<Vec<_>>>().map_err(Into::into)
}

fn split_type_to_str(t: SplitType) -> &'static str {
    match t {
        SplitType::ForwardSplit => "forward_split",
        SplitType::ReverseSplit => "reverse_split",
        SplitType::StockDividend => "stock_dividend",
        SplitType::Unknown => "unknown",
    }
}

fn str_to_split_type(s: Option<&str>) -> SplitType {
    match s {
        Some("forward_split") => SplitType::ForwardSplit,
        Some("reverse_split") => SplitType::ReverseSplit,
        Some("stock_dividend") => SplitType::StockDividend,
        _ => SplitType::Unknown,
    }
}
