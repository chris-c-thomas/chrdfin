//! Source-of-truth for the `source` column priority order.
//!
//! Higher numeric priority means "more authoritative". On upsert conflict
//! we only let the new row overwrite the existing one when the new source's
//! priority is greater than or equal to the old source's. That keeps a
//! manual bulk historical backfill (priority 100) safe from being clobbered
//! by an incremental Massive sync (priority 50).
//!
//! Phase 1 ships with only `manual_backfill` and `massive`. Add new
//! adapters here as they land — the SQL builder picks them up
//! automatically.

/// Ordered list of `(source_name, priority)` pairs. Order doesn't matter
/// for correctness — only the priority value does — but keep entries
/// grouped by tier for readability.
pub const SOURCE_PRIORITY: &[(&str, u8)] = &[
    // Hand-curated bulk imports (Parquet/CSV) beat anything fetched live.
    ("manual_backfill", 100),
    // Phase 1's only API adapter.
    ("massive", 50),
];

/// Return the priority for a source string, or `0` for unknown sources
/// (which is intentional — an unrecognized source can never overwrite a
/// recognized one).
pub fn priority(source: &str) -> u8 {
    SOURCE_PRIORITY
        .iter()
        .find(|(name, _)| *name == source)
        .map(|(_, prio)| *prio)
        .unwrap_or(0)
}

/// Build a SQL `CASE` expression that maps the named column or alias to
/// its priority value. Used inside `INSERT … ON CONFLICT … DO UPDATE …
/// WHERE` clauses so the priority comparison happens atomically with the
/// write. Returning a string fragment beats registering a DuckDB UDF —
/// no per-connection registration to worry about, and the priority table
/// stays in one Rust place.
///
/// `column` is interpolated raw — only ever pass a literal column or
/// alias reference (e.g. `"EXCLUDED.source"` or `"daily_prices.source"`).
/// Never pass user input.
pub fn priority_case_sql(column: &str) -> String {
    let mut out = String::with_capacity(64 + 32 * SOURCE_PRIORITY.len());
    out.push_str("CASE ");
    out.push_str(column);
    for (name, prio) in SOURCE_PRIORITY {
        out.push_str(&format!(" WHEN '{name}' THEN {prio}"));
    }
    out.push_str(" ELSE 0 END");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_sources_have_expected_priority() {
        assert_eq!(priority("manual_backfill"), 100);
        assert_eq!(priority("massive"), 50);
    }

    #[test]
    fn unknown_source_is_zero() {
        assert_eq!(priority("tiingo"), 0);
        assert_eq!(priority(""), 0);
    }

    #[test]
    fn case_sql_includes_every_known_source() {
        let sql = priority_case_sql("daily_prices.source");
        assert!(sql.contains("WHEN 'manual_backfill' THEN 100"));
        assert!(sql.contains("WHEN 'massive' THEN 50"));
        assert!(sql.starts_with("CASE daily_prices.source"));
        assert!(sql.ends_with(" ELSE 0 END"));
    }
}
