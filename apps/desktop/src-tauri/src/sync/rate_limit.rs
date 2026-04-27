use std::num::NonZeroU32;

use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};

use crate::secrets::MassiveTier;

/// Token-bucket rate limiter wrapper. Each provider instantiates one of these
/// and calls `until_ready().await` immediately before every outbound request.
///
/// The free tier's 5 RPM is the binding constraint that drove the orchestrator
/// down to single-request concurrency in sub-phase 1D — even one extra
/// request in flight would just queue behind the limiter.
pub struct ProviderRateLimiter {
    inner: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    label: &'static str,
}

impl ProviderRateLimiter {
    pub fn new(label: &'static str, quota: Quota) -> Self {
        Self {
            inner: RateLimiter::direct(quota),
            label,
        }
    }

    /// Build a limiter sized for the given Massive tier.
    ///
    /// - `Free`: 5 requests / minute. Bucket holds 5 tokens and replenishes
    ///   1 every 12 seconds — matches the documented free-tier cap.
    /// - `Paid`: 100 requests / second. Conservative starting point for
    ///   Stocks Starter and above; bump if a higher plan justifies it.
    ///
    /// Single-request concurrency on Free is enforced separately by the
    /// orchestrator (sub-phase 1D), not by the limiter.
    pub fn for_massive_tier(tier: MassiveTier) -> Self {
        let quota = match tier {
            MassiveTier::Free => Quota::per_minute(NonZeroU32::new(5).expect("5 != 0")),
            MassiveTier::Paid => Quota::per_second(NonZeroU32::new(100).expect("100 != 0")),
        };
        Self::new("massive", quota)
    }

    /// Block until the limiter says a request may be sent. Cheap to call
    /// when capacity is available; awaits only when the bucket is empty.
    pub async fn until_ready(&self) {
        self.inner.until_ready().await;
    }

    pub fn label(&self) -> &'static str {
        self.label
    }

    /// Non-blocking probe used by tests to verify capacity behavior without
    /// waiting on real wall-clock time (`governor` uses its own monotonic
    /// clock and ignores `tokio::time::pause`). Returns `Ok(())` if a token
    /// was available and consumed, `Err(())` if the bucket was empty.
    #[cfg(test)]
    fn try_acquire(&self) -> Result<(), ()> {
        self.inner.check().map_err(|_| ())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Massive's free tier must allow exactly 5 immediate acquisitions in
    /// a fresh window and reject the 6th. Uses `try_acquire` so the test
    /// is instant — `governor`'s clock is independent of `tokio::time::pause`,
    /// so a blocking-style test would burn ~12s of real wall-clock per token.
    #[test]
    fn massive_free_tier_caps_at_five_per_window() {
        let limiter = ProviderRateLimiter::for_massive_tier(MassiveTier::Free);

        for i in 0..5 {
            assert!(
                limiter.try_acquire().is_ok(),
                "request {i} of first 5 should succeed"
            );
        }

        assert!(
            limiter.try_acquire().is_err(),
            "6th request inside the same window should be rejected"
        );
    }

    /// Sanity check — paid tier's burst (100) easily absorbs more than the
    /// free tier's per-minute cap, so a tight loop that would block on Free
    /// breezes through here.
    #[test]
    fn paid_tier_absorbs_burst() {
        let limiter = ProviderRateLimiter::for_massive_tier(MassiveTier::Paid);
        for i in 0..50 {
            assert!(
                limiter.try_acquire().is_ok(),
                "paid burst should accept request {i}"
            );
        }
        assert_eq!(limiter.label(), "massive");
    }
}
