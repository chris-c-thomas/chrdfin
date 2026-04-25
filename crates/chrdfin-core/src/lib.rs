//! chrdfin-core — native computation engine.
//!
//! Phase 0 ships module stubs only. Each module is filled in by its
//! corresponding phase (see `docs/technical-blueprint.md`):
//!
//! - `backtest`     → Phase 2
//! - `monte_carlo`  → Phase 4
//! - `optimizer`    → Phase 9
//! - `stats`        → Phase 2
//! - `portfolio`    → Phase 2
//! - `matrix`       → Phase 9
//! - `calculators`  → Phase 6
//! - `types`        → cross-cutting

pub mod backtest;
pub mod calculators;
pub mod matrix;
pub mod monte_carlo;
pub mod optimizer;
pub mod portfolio;
pub mod stats;
pub mod types;

/// Health check used by the Tauri `health_check` command.
///
/// Returns the crate name + version; a successful round-trip from the
/// frontend confirms the compute engine is linked into the desktop app.
pub fn health_check() -> String {
    format!(
        "{} v{} loaded",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_check_includes_crate_name_and_version() {
        let s = health_check();
        assert!(s.contains("chrdfin-core"));
        assert!(s.contains(env!("CARGO_PKG_VERSION")));
    }
}
