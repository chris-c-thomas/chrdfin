//! Provider-agnostic data ingestion layer.
//!
//! Phase 1A lays only the foundation: a `DataProvider`/`MacroProvider` trait
//! pair, the shared error model, the per-provider rate limiter, and the
//! provider-side DTOs. Concrete adapters (1B), the orchestrator (1D), and the
//! background scheduler (1F) plug in on top.
//!
//! The module-wide `dead_code` allow exists because every item declared here
//! is referenced only from later sub-phases. Remove it in 1B once the
//! Massive adapter starts consuming the trait + types + error variants.
#![allow(dead_code)]

pub mod error;
pub mod provider;
pub mod rate_limit;
pub mod types;
