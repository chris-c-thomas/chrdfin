//! Free-tier caveats expressed as named constants. The orchestrator
//! (sub-phase 1D) reads these to clamp `Sync::run(Full)`'s start date and to
//! size its retry windows.

/// Maximum requests-per-minute on the Massive free tier. Mirrors the
/// `governor::Quota` used by `ProviderRateLimiter::for_massive_tier`.
pub const FREE_TIER_RPM: u32 = 5;

/// Approximate history depth available on the free tier for the aggregates
/// endpoint. Anything older than this is gated behind a paid plan and the
/// API returns a `NOT_AUTHORIZED` payload.
pub const FREE_TIER_HISTORY_YEARS: i64 = 2;
