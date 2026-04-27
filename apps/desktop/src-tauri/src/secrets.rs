use std::str::FromStr;

/// Provider-tier selector. Pinned via the `MASSIVE_TIER` env var; defaults to
/// `Free` when unset. Drives both the rate limiter (`Free` = 5 RPM) and the
/// orchestrator's history-clamp (`Free` = ~2 years on aggregates).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MassiveTier {
    #[default]
    Free,
    Paid,
}

impl FromStr for MassiveTier {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_lowercase().as_str() {
            "free" => Ok(Self::Free),
            // Accept the common plan names as synonyms for `Paid` so users
            // don't have to remember which token we picked.
            "paid" | "starter" | "developer" | "advanced" | "business" => Ok(Self::Paid),
            other => Err(format!("unknown MASSIVE_TIER: {other}")),
        }
    }
}

/// Snapshot of every API credential the app needs, taken once at startup.
///
/// A missing key is intentionally not a startup error — the app launches
/// regardless so the user can still browse data already in DuckDB. The
/// missing key surfaces as a typed `ProviderError::MissingApiKey` only when
/// a sync is actually attempted.
///
/// Field-level `dead_code` allow: both fields are read by the Massive
/// adapter (sub-phase 1B); 1A wires them in but doesn't consume them yet.
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Secrets {
    pub massive_api_key: Option<String>,
    pub massive_tier: MassiveTier,
}

impl Secrets {
    /// Read every credential from the process environment. Logs but does not
    /// fail when keys are absent or the tier value is unparseable.
    pub fn from_env() -> Self {
        let massive_api_key = std::env::var("MASSIVE_API_KEY")
            .ok()
            .map(|v| v.trim().to_owned())
            .filter(|v| !v.is_empty());

        if massive_api_key.is_none() {
            tracing::warn!(
                "MASSIVE_API_KEY is not set — sync will fail with MissingApiKey \
                 until a key is configured in .env.local"
            );
        }

        let massive_tier = match std::env::var("MASSIVE_TIER") {
            Ok(raw) => match MassiveTier::from_str(&raw) {
                Ok(tier) => tier,
                Err(err) => {
                    tracing::warn!(?err, "falling back to MassiveTier::Free");
                    MassiveTier::Free
                }
            },
            Err(_) => MassiveTier::Free,
        };

        tracing::info!(?massive_tier, "secrets loaded");

        Self {
            massive_api_key,
            massive_tier,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tier_defaults_to_free() {
        assert_eq!(MassiveTier::default(), MassiveTier::Free);
    }

    #[test]
    fn tier_parses_free() {
        assert_eq!("free".parse::<MassiveTier>().unwrap(), MassiveTier::Free);
        assert_eq!("FREE".parse::<MassiveTier>().unwrap(), MassiveTier::Free);
        assert_eq!(
            "  Free  ".parse::<MassiveTier>().unwrap(),
            MassiveTier::Free
        );
    }

    #[test]
    fn tier_parses_paid_and_synonyms() {
        for s in ["paid", "starter", "Developer", "ADVANCED", "business"] {
            assert_eq!(
                s.parse::<MassiveTier>().unwrap(),
                MassiveTier::Paid,
                "input: {s}"
            );
        }
    }

    #[test]
    fn tier_rejects_unknown() {
        assert!("enterprise".parse::<MassiveTier>().is_err());
    }
}
