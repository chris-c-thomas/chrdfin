//! Typed write/read helpers over DuckDB.
//!
//! Every adapter result lands here on the way in; every Tauri read command
//! pulls from here on the way out. Keeping all SQL inside this module makes
//! the storage shape easy to grep for (and easy to swap if the backend ever
//! moves off DuckDB — see `docs/technical-blueprint.md` "Storage backend
//! swap path").
//!
//! Module-wide `dead_code` allow stays in place: the orchestrator
//! (sub-phase 1D) is what first drives these helpers from production code.
//! Lift it in 1D.
#![allow(dead_code)]

pub mod assets;
pub mod dividends;
pub mod macros;
pub mod prices;
pub mod settings;
pub mod source;
pub mod splits;
pub mod sync_log;
