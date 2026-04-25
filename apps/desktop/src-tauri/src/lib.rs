mod commands;
mod db;
mod error;
mod state;

use tauri::Manager;

use crate::db::Database;
use crate::state::AppState;

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

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let db = Database::initialize(app.handle())?;
            app.manage(AppState::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::health_check,
            commands::system::get_theme,
            commands::system::set_theme,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
