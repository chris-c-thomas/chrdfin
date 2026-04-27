//! Massive (rebranded Polygon.io) data-provider adapter.
//!
//! Implements both `DataProvider` (equity reference, prices, dividends, splits)
//! and `MacroProvider` (treasury yields, inflation, labor market) against
//! `https://api.massive.com`. Auth is `Authorization: Bearer <key>`.
//!
//! Layout:
//! - [`client`] — the public `MassiveProvider` struct, the rate-limited GET
//!   helper, and the `DataProvider` + `MacroProvider` impls.
//! - [`dto`] — provider-internal `serde::Deserialize` shapes that mirror the
//!   raw JSON responses field-for-field.
//! - [`mappers`] — pure functions that convert the raw DTOs to the public
//!   types in `crate::sync::types`. Keeps decode and translation separable
//!   so unit tests can hit either layer in isolation.
//! - [`limits`] — named constants for the free-tier caveats so the
//!   orchestrator (sub-phase 1D) can branch on them rather than guess.

pub mod client;
pub mod dto;
pub mod limits;
pub mod mappers;
