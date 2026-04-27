use duckdb::{Connection, params};
use serde::{Serialize, de::DeserializeOwned};

use crate::error::{AppError, AppResult};

/// Read a typed setting. Returns `Ok(None)` when the key is absent.
pub fn get_setting<T: DeserializeOwned>(conn: &Connection, key: &str) -> AppResult<Option<T>> {
    let mut stmt = conn.prepare("SELECT value FROM app_settings WHERE key = ?")?;
    let mut rows = stmt.query(params![key])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    let raw: String = row.get(0)?;
    let parsed: T = serde_json::from_str(&raw)
        .map_err(|e| AppError::InvalidInput(format!("settings[{key}] decode: {e}")))?;
    Ok(Some(parsed))
}

/// Insert or update a typed setting. Atomic per call.
pub fn set_setting<T: Serialize>(conn: &Connection, key: &str, value: &T) -> AppResult<()> {
    let raw = serde_json::to_string(value)
        .map_err(|e| AppError::InvalidInput(format!("settings[{key}] encode: {e}")))?;
    conn.execute(
        r#"
        INSERT INTO app_settings (key, value)
        VALUES (?, ?)
        ON CONFLICT (key) DO UPDATE SET
            value = EXCLUDED.value,
            updated_at = now()
        "#,
        params![key, raw],
    )?;
    Ok(())
}
