//! Tauri commands that drive the sync orchestrator.
//!
//! `sync_data` is the single entry point used by both the manual
//! "Refresh data" UI button (sub-phase 1E) and the background scheduler
//! (sub-phase 1F). It runs the full or incremental orchestrator, emits
//! `sync:progress` events while in flight, and resolves with a
//! `SyncSummary`.
//!
//! `get_sync_status` is a small read used by the dashboard widget so the
//! UI can render "Last synced 2h ago" without standing up a polling loop
//! against the orchestrator itself.

use std::sync::Arc;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::storage::sync_log;
use crate::sync::orchestrator::{Sync, SyncMode, SyncProgress, SyncSummary};

const PROGRESS_EVENT: &str = "sync:progress";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDataInput {
    pub mode: SyncMode,
}

/// Run a full or incremental sync. Emits `sync:progress` events as the
/// orchestrator advances and resolves with the final `SyncSummary`.
///
/// Per-ticker errors are bundled into `summary.errors` rather than
/// failing the whole call — the frontend can decide how to surface them.
/// A hard failure (rate-limit ceiling, lock contention, DB error)
/// propagates as `Err(_)` to the JS side.
#[tauri::command]
pub async fn sync_data(
    app: AppHandle,
    state: State<'_, AppState>,
    input: SyncDataInput,
) -> AppResult<SyncSummary> {
    let sync: Arc<Sync> = state.sync.clone();
    let app_for_progress = app.clone();

    let on_progress: Box<dyn Fn(SyncProgress) + Send + std::marker::Sync> =
        Box::new(move |progress| {
            if let Err(err) = app_for_progress.emit(PROGRESS_EVENT, &progress) {
                tracing::warn!(?err, "failed to emit sync:progress");
            }
        });

    sync.run(input.mode, on_progress).await
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncStatus {
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub latest: Option<SyncRunRow>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncRunRow {
    pub id: String,
    pub sync_type: String,
    pub status: String,
    pub tickers_synced: Option<i32>,
    pub rows_upserted: Option<i32>,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Read-only status surface for the dashboard. Returns the latest
/// `sync_log` row plus the timestamp of the most recent successful run
/// so the UI can render "Last synced N minutes ago" alongside any
/// in-progress or failed indicator.
#[tauri::command]
pub fn get_sync_status(state: State<'_, AppState>) -> AppResult<SyncStatus> {
    let conn = state
        .db
        .conn
        .lock()
        .map_err(|e| AppError::InvalidInput(format!("db mutex poisoned: {e}")))?;

    let last_successful_sync = sync_log::last_successful_sync(&conn)?;

    let mut stmt = conn.prepare(
        r#"
        SELECT id, sync_type, status, tickers_synced, rows_upserted,
               error_message, started_at, completed_at
        FROM sync_log
        ORDER BY started_at DESC
        LIMIT 1
        "#,
    )?;
    let mut rows = stmt.query([])?;
    let latest = if let Some(row) = rows.next()? {
        Some(SyncRunRow {
            id: row.get(0)?,
            sync_type: row.get(1)?,
            status: row.get(2)?,
            tickers_synced: row.get(3)?,
            rows_upserted: row.get(4)?,
            error_message: row.get(5)?,
            started_at: row.get(6)?,
            completed_at: row.get(7)?,
        })
    } else {
        None
    };

    Ok(SyncStatus {
        last_successful_sync,
        latest,
    })
}
