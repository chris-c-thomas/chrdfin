use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub db_initialized: bool,
    pub version: String,
    pub core_version: String,
}

/// Round-trip command used by the dashboard and CI smoke tests to confirm
/// the IPC plumbing and DuckDB initialization both work.
#[tauri::command]
pub fn health_check(state: State<'_, AppState>) -> AppResult<HealthCheckResponse> {
    // Perform a trivial DB read to confirm the connection is alive.
    let conn = state
        .db
        .conn
        .lock()
        .map_err(|e| AppError::InvalidInput(format!("db mutex poisoned: {e}")))?;
    let table_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'main'",
        [],
        |row| row.get(0),
    )?;

    Ok(HealthCheckResponse {
        status: format!("ok ({table_count} tables)"),
        db_initialized: true,
        version: env!("CARGO_PKG_VERSION").to_string(),
        core_version: chrdfin_core::health_check(),
    })
}

#[derive(Debug, Deserialize)]
pub struct SetThemeArgs {
    pub theme: String,
}

/// Read the persisted theme preference from `app_settings`.
/// Returns "dark" if no value has been stored yet.
#[tauri::command]
pub fn get_theme(state: State<'_, AppState>) -> AppResult<String> {
    let conn = state
        .db
        .conn
        .lock()
        .map_err(|e| AppError::InvalidInput(format!("db mutex poisoned: {e}")))?;

    let stored: Option<String> = conn
        .query_row(
            "SELECT value::VARCHAR FROM app_settings WHERE key = 'theme'",
            [],
            |row| row.get::<_, String>(0),
        )
        .ok();

    Ok(stored
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|| "dark".to_string()))
}

/// Persist the theme preference to `app_settings`. Theme must be one of
/// "light", "dark", or "system".
#[tauri::command]
pub fn set_theme(args: SetThemeArgs, state: State<'_, AppState>) -> AppResult<()> {
    let theme = args.theme;
    if !matches!(theme.as_str(), "light" | "dark" | "system") {
        return Err(AppError::InvalidInput(format!(
            "invalid theme: {theme}; expected light|dark|system"
        )));
    }

    let conn = state
        .db
        .conn
        .lock()
        .map_err(|e| AppError::InvalidInput(format!("db mutex poisoned: {e}")))?;

    let json = serde_json::to_string(&theme)?;
    conn.execute(
        "INSERT INTO app_settings (key, value) VALUES ('theme', ?::JSON)
         ON CONFLICT (key) DO UPDATE SET value = excluded.value, updated_at = current_timestamp",
        [json],
    )?;
    Ok(())
}
