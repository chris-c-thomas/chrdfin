use chrono::NaiveDate;
use duckdb::{Connection, params};

use crate::error::AppResult;
use crate::sync::types::{MacroObservation, MacroSeriesId};

use super::source::priority_case_sql;

pub fn upsert_macro_observations(
    conn: &Connection,
    rows: &[MacroObservation],
    source: &str,
) -> AppResult<usize> {
    if rows.is_empty() {
        return Ok(0);
    }

    let sql = format!(
        r#"
        INSERT INTO macro_series (series_id, date, value, source)
        VALUES (?, ?, ?, ?)
        ON CONFLICT (series_id, date) DO UPDATE SET
            value  = EXCLUDED.value,
            source = EXCLUDED.source
        WHERE {new} >= {old}
        "#,
        new = priority_case_sql("EXCLUDED.source"),
        old = priority_case_sql("macro_series.source"),
    );

    conn.execute_batch("BEGIN TRANSACTION")?;
    let mut total = 0usize;
    {
        let mut stmt = conn.prepare(&sql)?;
        for o in rows {
            total += stmt.execute(params![o.series.as_db_str(), o.date, o.value, source,])?;
        }
    }
    conn.execute_batch("COMMIT")?;
    Ok(total)
}

pub fn get_macro_series(
    conn: &Connection,
    series: MacroSeriesId,
    start: NaiveDate,
    end: NaiveDate,
) -> AppResult<Vec<MacroObservation>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT series_id, date, value
        FROM macro_series
        WHERE series_id = ? AND date BETWEEN ? AND ?
        ORDER BY date ASC
        "#,
    )?;
    let rows = stmt.query_map(params![series.as_db_str(), start, end], |row| {
        let series_str: String = row.get(0)?;
        Ok(MacroObservation {
            series: MacroSeriesId::from_db_str(&series_str)
                // Fall back to the requested series if a row was written
                // under an alias we no longer recognize (defensive — should
                // not happen with the closed enum).
                .unwrap_or(series),
            date: row.get(1)?,
            value: row.get(2)?,
        })
    })?;
    rows.collect::<duckdb::Result<Vec<_>>>().map_err(Into::into)
}

pub fn latest_macro_date(conn: &Connection, series: MacroSeriesId) -> AppResult<Option<NaiveDate>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT max(date) FROM macro_series WHERE series_id = ?
        "#,
    )?;
    let mut rows = stmt.query(params![series.as_db_str()])?;
    if let Some(row) = rows.next()? {
        Ok(row.get::<_, Option<NaiveDate>>(0)?)
    } else {
        Ok(None)
    }
}
