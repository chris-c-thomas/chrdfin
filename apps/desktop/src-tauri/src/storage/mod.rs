//! Typed write/read helpers over DuckDB.
//!
//! Every adapter result lands here on the way in; every Tauri read command
//! pulls from here on the way out. Keeping all SQL inside this module makes
//! the storage shape easy to grep for (and easy to swap if the backend ever
//! moves off DuckDB — see `docs/technical-blueprint.md` "Storage backend
//! swap path").

pub mod assets;
pub mod dividends;
pub mod macros;
pub mod prices;
pub mod settings;
pub mod source;
pub mod splits;
pub mod sync_log;
