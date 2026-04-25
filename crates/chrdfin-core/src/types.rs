//! Shared types used across compute modules.
//!
//! Mirrors the TypeScript shape in `@chrdfin/types` for cross-boundary
//! serialization. Phase 0 surfaces only the bare minimum needed by the
//! `health_check` command and by future module signatures.

use serde::{Deserialize, Serialize};

/// Annualized statistics about a return series. Filled out in Phase 2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioMetrics {
    pub total_return: f64,
    pub cagr: f64,
    pub annualized_volatility: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub calmar_ratio: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub best_year: f64,
    pub worst_year: f64,
    pub var_95: f64,
    pub cvar_95: f64,
    pub win_rate: f64,
    pub ulcer_index: f64,
}

/// Progress event reported during long-running computations. Tauri's
/// command handlers forward these as window events to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub phase: String,
    pub current: u64,
    pub total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Domain error type. Each module adds variants as it ships.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("not implemented: {0}")]
    NotImplemented(&'static str),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("computation failed: {0}")]
    Computation(String),
}

pub type CoreResult<T> = Result<T, CoreError>;
