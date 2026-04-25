use crate::db::Database;

/// Application state stored as a Tauri-managed resource. Accessed from
/// command handlers via `tauri::State<'_, AppState>`.
pub struct AppState {
    pub db: Database,
}

impl AppState {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}
