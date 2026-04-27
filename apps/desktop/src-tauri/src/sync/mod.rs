//! Provider-agnostic data ingestion layer.
//!
//! Phase 1A laid the foundation (trait pair, shared error model, rate
//! limiter, provider-side DTOs). Phase 1B added `MassiveProvider`. Phase
//! 1D wires it all into the `orchestrator`, which is what production
//! code drives. Background scheduler (1F) plugs in on top.

pub mod error;
pub mod massive;
pub mod orchestrator;
pub mod provider;
pub mod rate_limit;
pub mod types;
