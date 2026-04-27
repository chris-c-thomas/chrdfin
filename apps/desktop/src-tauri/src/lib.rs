// `pub` so the integration tests under `tests/` can reach the adapter, types,
// shared infrastructure, and the `*_inner` test entry points on the commands.
// The Tauri command handlers themselves live inside the crate and would work
// either way.
pub mod commands;
pub mod db;
mod error;
pub mod http;
pub mod secrets;
mod state;
pub mod storage;
pub mod sync;

use std::sync::Arc;

use tauri::Manager;

use crate::db::Database;
use crate::http::AppHttpClient;
use crate::secrets::Secrets;
use crate::state::AppState;
use crate::sync::massive::client::MassiveProvider;
use crate::sync::orchestrator::Sync;
use crate::sync::scheduler::spawn_background_sync;

/// Tauri application entry point — invoked by both `main.rs` and the
/// mobile/desktop launcher in production builds.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let http = AppHttpClient::new();
    let secrets = Secrets::from_env();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(move |app| {
            let db = Arc::new(Database::initialize(app.handle())?);
            let massive = Arc::new(MassiveProvider::new(
                http.clone(),
                secrets.massive_api_key.clone(),
                secrets.massive_tier,
            ));
            let sync = Arc::new(Sync::new(massive, db.clone(), secrets.massive_tier));
            // Detached background task — Tauri's tokio runtime owns it
            // for the app's lifetime.
            let _scheduler_handle = spawn_background_sync(sync.clone(), db.clone());
            app.manage(AppState::new(db, http, secrets, sync));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::health_check,
            commands::system::get_theme,
            commands::system::set_theme,
            commands::sync::sync_data,
            commands::sync::get_sync_status,
            commands::sync::get_recent_sync_runs,
            commands::data::get_prices,
            commands::data::get_macro_series,
            commands::data::get_asset_metadata,
            commands::data::search_tickers,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
