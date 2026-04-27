//! Provider-agnostic data ingestion layer.
//!
//! Phase 1A laid the foundation (trait pair, shared error model, rate
//! limiter, provider-side DTOs). Phase 1B's `MassiveProvider` is the first
//! concrete adapter. The orchestrator (1D) and background scheduler (1F)
//! plug in on top.
//!
//! The module-wide `dead_code` allow stays in place because the adapter has
//! no production caller yet — the orchestrator (sub-phase 1D) is what first
//! invokes `DataProvider`/`MacroProvider` outside of tests. Lift this when
//! the orchestrator lands.
#![allow(dead_code)]

pub mod error;
pub mod massive;
pub mod provider;
pub mod rate_limit;
pub mod types;
