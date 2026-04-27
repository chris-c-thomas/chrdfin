use chrono::NaiveDate;
use duckdb::{Connection, params};

use crate::error::AppResult;
use crate::sync::types::{Dividend, DividendType};

use super::source::priority_case_sql;

pub fn upsert_dividends(conn: &Connection, rows: &[Dividend], source: &str) -> AppResult<usize> {
    if rows.is_empty() {
        return Ok(0);
    }

    let sql = format!(
        r#"
        INSERT INTO dividends (ticker, ex_date, amount, div_type, source)
        VALUES (?, ?, ?, ?, ?)
        ON CONFLICT (ticker, ex_date) DO UPDATE SET
            amount   = EXCLUDED.amount,
            div_type = EXCLUDED.div_type,
            source   = EXCLUDED.source
        WHERE {new} >= {old}
        "#,
        new = priority_case_sql("EXCLUDED.source"),
        old = priority_case_sql("dividends.source"),
    );

    conn.execute_batch("BEGIN TRANSACTION")?;
    let mut total = 0usize;
    {
        let mut stmt = conn.prepare(&sql)?;
        for d in rows {
            total += stmt.execute(params![
                d.ticker,
                d.ex_date,
                d.amount,
                div_type_to_str(d.div_type),
                source,
            ])?;
        }
    }
    conn.execute_batch("COMMIT")?;
    Ok(total)
}

pub fn get_dividends(
    conn: &Connection,
    ticker: &str,
    start: NaiveDate,
    end: NaiveDate,
) -> AppResult<Vec<Dividend>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT ticker, ex_date, amount, div_type
        FROM dividends
        WHERE ticker = ? AND ex_date BETWEEN ? AND ?
        ORDER BY ex_date ASC
        "#,
    )?;
    let rows = stmt.query_map(params![ticker, start, end], |row| {
        let div_type_str: Option<String> = row.get(3)?;
        Ok(Dividend {
            ticker: row.get(0)?,
            ex_date: row.get(1)?,
            amount: row.get(2)?,
            div_type: str_to_div_type(div_type_str.as_deref()),
        })
    })?;
    rows.collect::<duckdb::Result<Vec<_>>>().map_err(Into::into)
}

fn div_type_to_str(t: DividendType) -> &'static str {
    match t {
        DividendType::Regular => "regular",
        DividendType::Special => "special",
        DividendType::ReturnOfCapital => "return_of_capital",
        DividendType::Unknown => "unknown",
    }
}

fn str_to_div_type(s: Option<&str>) -> DividendType {
    match s {
        Some("regular") => DividendType::Regular,
        Some("special") => DividendType::Special,
        Some("return_of_capital") => DividendType::ReturnOfCapital,
        _ => DividendType::Unknown,
    }
}
