use crate::db::Database;
use crate::http::AppHttpClient;
use crate::secrets::Secrets;

/// Application state stored as a Tauri-managed resource. Accessed from
/// command handlers via `tauri::State<'_, AppState>`.
///
/// `http` and `secrets` are wired here in 1A so the rest of the data layer
/// can grow against a stable shape. The sync orchestrator (1D) and command
/// handlers begin reading them in subsequent sub-phases.
pub struct AppState {
    pub db: Database,
    #[allow(dead_code)]
    pub http: AppHttpClient,
    #[allow(dead_code)]
    pub secrets: Secrets,
}

impl AppState {
    pub fn new(db: Database, http: AppHttpClient, secrets: Secrets) -> Self {
        Self { db, http, secrets }
    }
}
