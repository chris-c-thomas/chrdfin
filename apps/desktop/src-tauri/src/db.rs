use std::sync::Mutex;

use duckdb::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{AppError, AppResult};

const SCHEMA_SQL: &str = include_str!("schema.sql");

/// Embedded DuckDB instance owned by the desktop app.
///
/// Wrapped in a `Mutex` for shared mutable access across Tauri command
/// handlers. DuckDB's own concurrency story is single-writer / many-reader,
/// and the desktop app is single-user, so a plain mutex is sufficient.
///
/// Callers must NEVER hold the mutex across an `.await` point — it's a
/// blocking `std::sync::Mutex`, and holding it across a yield would block
/// every other task that needs DB access. The orchestrator's pattern is:
/// fetch async, then briefly lock + write + drop the guard before the
/// next await.
pub struct Database {
    pub conn: Mutex<Connection>,
}

impl Database {
    /// Initialize the on-disk DuckDB at the platform's app data dir,
    /// creating the directory if needed and applying the schema.
    pub fn initialize(app: &AppHandle) -> AppResult<Self> {
        let data_dir = app
            .path()
            .app_data_dir()
            .map_err(|e| AppError::InvalidInput(format!("failed to resolve app data dir: {e}")))?;
        std::fs::create_dir_all(&data_dir)?;
        let db_path = data_dir.join("chrdfin.duckdb");

        tracing::info!(?db_path, "opening DuckDB");
        let conn = Connection::open(&db_path)?;
        conn.execute_batch(SCHEMA_SQL)?;
        tracing::info!("DuckDB schema applied");

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Test/integration helper that wraps an already-prepared `Connection`
    /// (typically an in-memory DuckDB with the schema pre-applied) so the
    /// orchestrator can be exercised against deterministic state.
    pub fn from_connection(conn: Connection) -> Self {
        Self {
            conn: Mutex::new(conn),
        }
    }
}
