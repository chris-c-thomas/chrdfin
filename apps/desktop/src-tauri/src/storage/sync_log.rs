use chrono::{DateTime, Utc};
use duckdb::{Connection, params};

use crate::error::AppResult;

/// Begin a sync_log row in the `started` state, returning its id so the
/// orchestrator can finalize it later.
pub fn start_sync(conn: &Connection, sync_type: &str) -> AppResult<String> {
    let id: String = conn.query_row(
        r#"
        INSERT INTO sync_log (sync_type, status, started_at)
        VALUES (?, 'started', now())
        RETURNING id
        "#,
        params![sync_type],
        |row| row.get(0),
    )?;
    Ok(id)
}

pub fn complete_sync(
    conn: &Connection,
    id: &str,
    tickers_synced: u32,
    rows_upserted: u32,
) -> AppResult<()> {
    conn.execute(
        r#"
        UPDATE sync_log
        SET status = 'completed',
            tickers_synced = ?,
            rows_upserted = ?,
            completed_at = now()
        WHERE id = ?
        "#,
        params![tickers_synced as i64, rows_upserted as i64, id],
    )?;
    Ok(())
}

pub fn fail_sync(conn: &Connection, id: &str, error_message: &str) -> AppResult<()> {
    conn.execute(
        r#"
        UPDATE sync_log
        SET status = 'failed',
            error_message = ?,
            completed_at = now()
        WHERE id = ?
        "#,
        params![error_message, id],
    )?;
    Ok(())
}

pub fn last_successful_sync(conn: &Connection) -> AppResult<Option<DateTime<Utc>>> {
    let mut stmt = conn.prepare(
        r#"
        SELECT max(completed_at) FROM sync_log WHERE status = 'completed'
        "#,
    )?;
    let mut rows = stmt.query([])?;
    if let Some(row) = rows.next()? {
        Ok(row.get::<_, Option<DateTime<Utc>>>(0)?)
    } else {
        Ok(None)
    }
}
