//! Background sync scheduler.
//!
//! Spawns a single tokio task on app launch that:
//! 1. Runs `Sync::seed_if_needed` so a fresh install populates the
//!    starter universe before any UI flow needs data.
//! 2. If `last_successful_sync` is missing or older than 24 hours,
//!    immediately runs an incremental sync.
//! 3. Loops forever, sleeping until the next 6:00 PM America/New_York
//!    on a weekday and running an incremental sync. DST transitions are
//!    handled by `chrono-tz`. Failures back off (1 hour, max 4 retries)
//!    before falling through to the next-day cadence.
//!
//! Manual `sync_data` invocations and the scheduled runs share the same
//! `Sync::run` mutex, so they never overlap.

use std::sync::Arc;
use std::time::Duration;

use chrono::{DateTime, Datelike, Duration as ChronoDuration, TimeZone, Utc, Weekday};
use chrono_tz::America::New_York;
use chrono_tz::Tz;
use tokio::task::JoinHandle;

use crate::db::Database;
use crate::storage::sync_log;
use crate::sync::orchestrator::{Sync, SyncMode};

/// 6 PM ET — chosen to land after the U.S. close (4 PM ET) plus 90
/// minutes of after-hours so end-of-day adjustments have settled.
const RUN_HOUR_ET: u32 = 18;

/// Threshold for an "automatic catch-up" on launch. If the last
/// successful sync is older than this, we kick an incremental sync
/// immediately rather than waiting for the next 6 PM ET tick.
const STALE_THRESHOLD: ChronoDuration = ChronoDuration::hours(24);

/// Backoff between failed runs.
const FAILURE_BACKOFF: ChronoDuration = ChronoDuration::hours(1);
const MAX_FAILURE_RETRIES: u32 = 4;

/// Spawn the background sync task. Returns the join handle so the
/// caller can keep it alive (or, in tests, abort it). The task is
/// detached in production — Tauri's runtime owns it for the app's
/// lifetime.
pub fn spawn_background_sync(sync: Arc<Sync>, db: Arc<Database>) -> JoinHandle<()> {
    tokio::spawn(async move {
        if let Err(err) = sync.seed_if_needed().await {
            tracing::warn!(?err, "seed_if_needed failed; continuing into scheduler");
        }

        if launch_catchup_due(&db) {
            tracing::info!("launch catch-up: kicking incremental sync");
            if let Err(err) = sync.run(SyncMode::Incremental, Box::new(|_| {})).await {
                tracing::warn!(?err, "launch catch-up sync failed");
            }
        }

        let mut failure_count: u32 = 0;
        loop {
            let now_utc = Utc::now();
            let next_utc = if failure_count > 0 && failure_count <= MAX_FAILURE_RETRIES {
                now_utc + FAILURE_BACKOFF
            } else {
                if failure_count > MAX_FAILURE_RETRIES {
                    tracing::warn!("max retries reached; falling back to daily cadence");
                    failure_count = 0;
                }
                next_run_at(now_utc, &New_York)
            };

            let wait = (next_utc - Utc::now())
                .to_std()
                .unwrap_or(Duration::from_secs(60));
            tracing::info!(?next_utc, ?wait, "sleeping until next scheduled sync");
            tokio::time::sleep(wait).await;

            tracing::info!("running scheduled incremental sync");
            match sync.run(SyncMode::Incremental, Box::new(|_| {})).await {
                Ok(_) => {
                    failure_count = 0;
                }
                Err(err) => {
                    failure_count = failure_count.saturating_add(1);
                    tracing::warn!(?err, failure_count, "scheduled sync failed");
                }
            }
        }
    })
}

/// Returns true when the last successful sync is missing or older than
/// `STALE_THRESHOLD`. DB-error path errs on the side of "yes" so a
/// transient lock failure on launch doesn't suppress the catch-up.
fn launch_catchup_due(db: &Database) -> bool {
    let conn = match db.conn.lock() {
        Ok(c) => c,
        Err(_) => return true,
    };
    match sync_log::last_successful_sync(&conn) {
        Ok(Some(ts)) => Utc::now() - ts > STALE_THRESHOLD,
        Ok(None) => true,
        Err(err) => {
            tracing::warn!(?err, "last_successful_sync read failed; assuming stale");
            true
        }
    }
}

/// Compute the next "6 PM America/New_York on a weekday" instant
/// strictly after `now`. DST-aware via `chrono-tz`.
///
/// Pure — no I/O, no globals. The scheduler tests pin specific UTC
/// instants and assert the returned timestamp lands where expected.
pub fn next_run_at(now: DateTime<Utc>, tz: &Tz) -> DateTime<Utc> {
    let now_local = now.with_timezone(tz);

    // First candidate: today at 18:00 local.
    let mut candidate = local_six_pm(now_local.date_naive(), tz);

    if candidate <= now {
        candidate = local_six_pm(now_local.date_naive() + ChronoDuration::days(1), tz);
    }

    // Skip Saturday + Sunday.
    while is_weekend(candidate.with_timezone(tz).weekday()) {
        candidate = local_six_pm(
            candidate.with_timezone(tz).date_naive() + ChronoDuration::days(1),
            tz,
        );
    }

    candidate
}

fn is_weekend(w: Weekday) -> bool {
    matches!(w, Weekday::Sat | Weekday::Sun)
}

/// Materialize 18:00:00 in `tz` on the given local date as a UTC
/// instant. On the spring-forward DST gap (no 2 AM exists, but 18:00 is
/// well clear of the gap), `single()` is the well-defined choice; on
/// fall-back (no gap at 6 PM either) likewise. We unwrap because 18:00
/// can never land in either ambiguity window for any real-world tz.
fn local_six_pm(date: chrono::NaiveDate, tz: &Tz) -> DateTime<Utc> {
    let local = date
        .and_hms_opt(RUN_HOUR_ET, 0, 0)
        .expect("18:00 is always a valid wall time");
    tz.from_local_datetime(&local)
        .single()
        .expect("18:00 is never inside a DST gap or fold")
        .with_timezone(&Utc)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Timelike;

    fn ny(date: (i32, u32, u32), time: (u32, u32, u32)) -> DateTime<Utc> {
        let (y, m, d) = date;
        let (h, min, s) = time;
        New_York
            .with_ymd_and_hms(y, m, d, h, min, s)
            .single()
            .expect("unambiguous local time")
            .with_timezone(&Utc)
    }

    #[test]
    fn weekday_before_6pm_returns_today_at_6pm() {
        // Wednesday 2026-04-22 10:00 ET → Wednesday 2026-04-22 18:00 ET
        let now = ny((2026, 4, 22), (10, 0, 0));
        let next = next_run_at(now, &New_York);
        assert_eq!(next, ny((2026, 4, 22), (18, 0, 0)));
    }

    #[test]
    fn weekday_after_6pm_returns_tomorrow_at_6pm() {
        // Wednesday 2026-04-22 19:00 ET → Thursday 2026-04-23 18:00 ET
        let now = ny((2026, 4, 22), (19, 0, 0));
        let next = next_run_at(now, &New_York);
        assert_eq!(next, ny((2026, 4, 23), (18, 0, 0)));
    }

    #[test]
    fn weekday_exactly_6pm_returns_tomorrow_at_6pm() {
        // Edge case: equality counts as "in the past" so we don't return
        // a zero-second wait.
        let now = ny((2026, 4, 22), (18, 0, 0));
        let next = next_run_at(now, &New_York);
        assert_eq!(next, ny((2026, 4, 23), (18, 0, 0)));
    }

    #[test]
    fn friday_after_6pm_skips_to_monday() {
        // Friday 2026-04-24 19:00 ET → Monday 2026-04-27 18:00 ET
        let now = ny((2026, 4, 24), (19, 0, 0));
        let next = next_run_at(now, &New_York);
        assert_eq!(next, ny((2026, 4, 27), (18, 0, 0)));
    }

    #[test]
    fn saturday_morning_skips_to_monday() {
        let now = ny((2026, 4, 25), (10, 0, 0));
        let next = next_run_at(now, &New_York);
        assert_eq!(next, ny((2026, 4, 27), (18, 0, 0)));
    }

    #[test]
    fn sunday_morning_skips_to_monday() {
        let now = ny((2026, 4, 26), (10, 0, 0));
        let next = next_run_at(now, &New_York);
        assert_eq!(next, ny((2026, 4, 27), (18, 0, 0)));
    }

    #[test]
    fn dst_spring_forward_keeps_6pm_local() {
        // 2026 spring-forward is Sunday 2026-03-08 at 02:00 ET.
        // Friday 2026-03-06 19:00 ET (still EST) → Monday 2026-03-09
        // 18:00 ET (now EDT). 18:00 is well clear of the 02:00 gap.
        let now = ny((2026, 3, 6), (19, 0, 0));
        let next = next_run_at(now, &New_York);
        let next_local = next.with_timezone(&New_York);
        assert_eq!(
            next_local.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 3, 9).unwrap()
        );
        assert_eq!(next_local.hour(), 18);
        assert_eq!(next_local.minute(), 0);
    }

    #[test]
    fn dst_fall_back_keeps_6pm_local() {
        // 2026 fall-back is Sunday 2026-11-01 at 02:00 ET.
        // Friday 2026-10-30 19:00 ET (EDT) → Monday 2026-11-02 18:00 ET
        // (now EST). 18:00 clears the 01:00–02:00 fold cleanly.
        let now = ny((2026, 10, 30), (19, 0, 0));
        let next = next_run_at(now, &New_York);
        let next_local = next.with_timezone(&New_York);
        assert_eq!(
            next_local.date_naive(),
            chrono::NaiveDate::from_ymd_opt(2026, 11, 2).unwrap()
        );
        assert_eq!(next_local.hour(), 18);
        assert_eq!(next_local.minute(), 0);
    }
}
