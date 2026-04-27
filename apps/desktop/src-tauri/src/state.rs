use std::sync::Arc;

use crate::db::Database;
use crate::http::AppHttpClient;
use crate::secrets::Secrets;
use crate::sync::orchestrator::Sync;

/// Application state stored as a Tauri-managed resource. Accessed from
/// command handlers via `tauri::State<'_, AppState>`.
///
/// `db` and `sync` are wrapped in `Arc` so the orchestrator and command
/// handlers can hold long-lived references without contending with Tauri's
/// own borrow of the managed state.
pub struct AppState {
    pub db: Arc<Database>,
    #[allow(dead_code)]
    pub http: AppHttpClient,
    #[allow(dead_code)]
    pub secrets: Secrets,
    pub sync: Arc<Sync>,
}

impl AppState {
    pub fn new(db: Arc<Database>, http: AppHttpClient, secrets: Secrets, sync: Arc<Sync>) -> Self {
        Self {
            db,
            http,
            secrets,
            sync,
        }
    }
}
