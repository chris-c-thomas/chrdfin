# Phase 1: Data Layer — Implementation Checklist

**Goal:** A working data ingestion and query layer powered by **Massive** (the rebranded Polygon.io). The Rust backend can fetch historical EOD prices, ticker reference data, dividends, splits, and Federal Reserve macro series; persist them to DuckDB; and serve them back to the frontend through typed Tauri commands. A background sync keeps the local store fresh after market close. The frontend can trigger a manual sync, observe progress in real time, and read results out of cache.

**Provider strategy.** Massive is the **sole** equity and macro provider for Phase 1 — no Tiingo, no FRED, no Polygon (Polygon **is** Massive — it rebranded on 2025-10-30, and `api.massive.com` is now the canonical base URL). The architecture stays multi-provider-friendly: a `DataProvider` trait, a `source` column on every persisted row, and a cascading-fallback pattern that can be extended later by adding new adapters without touching call sites. **Trading-broker adapters (Schwab, Tradier, etc.) are explicitly out of scope** — they are part of the post-v1.0 Trading roadmap.

**Bulk historical backfill is deferred.** The user will provide a bulk historical dataset (30+ years EOD) at the appropriate time, loaded directly into DuckDB via Parquet or COPY. Phase 1 implements only the steady-state incremental sync against Massive's free tier, which is sufficient for the MVP. The `source` column ensures backfilled rows can coexist with API-fetched rows.

**Out of scope (deferred):**
- Real-time quote polling and the `get_quotes` command (Phase 5 — Tracker; note that Massive free tier is EOD-only, so Phase 5 will likely require a plan upgrade or fallback to last-close pricing).
- Options chains (Phase 7 — Market Data).
- News and RSS feeds (Phase 8).
- Earnings calendar (Phase 8).
- OS-keychain API key storage (Phase 10 — Polish).
- Bulk historical seed (timing TBD; user-provided dataset).
- Trading broker integrations (post-v1.0).

**Sub-phase structure.** Phase 1 ships as **six sub-phases (1A–1F)**, each landing as its own commit on the `feat/phase-1` branch. The user runs `git commit` (signed) themselves; Claude drafts the message at the end of each sub-phase. **One PR is opened only after 1F is complete**, covering the entire Phase 1 scope.

**Reference reading before starting:** `docs/agent-handoff.md`, `CLAUDE.md`, `docs/technical-blueprint.md` (DR-002, DR-003, "Data Acquisition & Storage", "Command Organization"), `docs/database-schema-reference.md`, and the live Massive docs (https://massive.com/docs) — supplement with the `mcp__massive__*` tools when verifying endpoint shapes.

---

## Massive endpoint inventory used in Phase 1

| Concern | Endpoint | Method on adapter |
|---|---|---|
| Historical daily bars | `GET /v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from}/{to}` | `fetch_prices` |
| Previous trading day | `GET /v2/aggs/ticker/{ticker}/prev` | `fetch_previous_close` |
| Ticker reference list | `GET /v3/reference/tickers` (paginated, `search`, `type`, `active` filters) | `search_tickers` |
| Single ticker overview | `GET /v3/reference/tickers/{ticker}` | `fetch_metadata` |
| Cash dividends | `GET /stocks/v1/dividends?ticker=...&ex_dividend_date.gte=...` | `fetch_dividends` |
| Splits | `GET /stocks/v1/splits?ticker=...&execution_date.gte=...` | `fetch_splits` |
| Treasury yields | `GET /fed/v1/treasury-yields?date.gte=...` | `fetch_treasury_yields` |
| Inflation indicators | `GET /fed/v1/inflation?date.gte=...` | `fetch_inflation` |
| Market status | `GET /v1/marketstatus/now` | `fetch_market_status` |
| Market holidays | `GET /v1/marketstatus/upcoming` | `fetch_market_holidays` |

**Free-tier caveats** (assume Polygon's documented free-tier behavior carries over until proven otherwise):
- ~5 requests per minute. Rate limiter must serialize requests, not parallelize them.
- ~2 years of history on aggregates. Anything older than that comes from the deferred bulk backfill, not from Massive.
- EOD only — no real-time WebSockets, no intraday aggregates beyond delayed.
- These caveats live in `chrdfin_desktop::sync::massive::limits` as named constants so the orchestrator can adjust gracefully when an upgraded key is configured.

**Auth:** `Authorization: Bearer <api-key>` header. Base URL: `https://api.massive.com`.

---

## Sub-phase 1A: Provider foundation

Lay the groundwork that every adapter will sit on top of. No DB writes, no commands yet.

### Files to add

- `apps/desktop/src-tauri/src/http.rs` — `AppHttpClient` newtype around `reqwest::Client`. 30s timeout, gzip on, redirect cap 5, user-agent `chrdfin/0.1.0 (+https://github.com/chris-c-thomas/chrdfin)`.
- `apps/desktop/src-tauri/src/secrets.rs` — reads `MASSIVE_API_KEY` from the environment at startup. Stored on `AppState` as `Option<String>`. Missing key surfaces as a typed `ProviderError::MissingApiKey` only when a sync is attempted; the app launches cleanly without it.
- `apps/desktop/src-tauri/src/sync/mod.rs` — module root.
- `apps/desktop/src-tauri/src/sync/types.rs` — provider-side DTOs (`DailyPrice`, `Dividend`, `Split`, `AssetMetadata`, `TickerSearchHit`, `MacroObservation`, `MacroSeriesId`).
- `apps/desktop/src-tauri/src/sync/error.rs` — `ProviderError` and `ProviderResult`.
- `apps/desktop/src-tauri/src/sync/rate_limit.rs` — `governor`-backed limiter, configured per-provider.
- `apps/desktop/src-tauri/src/sync/provider.rs` — the `DataProvider` and `MacroProvider` traits.

### `Cargo.toml` additions (`apps/desktop/src-tauri`)

```toml
reqwest = { version = "0.12", default-features = false, features = ["rustls-tls", "json", "gzip", "stream"] }
url = "2"
async-trait = "0.1"
governor = "0.7"
futures = "0.3"
once_cell = "1"
```

> `default-features = false` + `rustls-tls` keeps the build off OpenSSL and matches the CI disk-size constraint already in place.

### Provider trait

```rust
// apps/desktop/src-tauri/src/sync/provider.rs

use async_trait::async_trait;
use chrono::NaiveDate;

use super::error::ProviderResult;
use super::types::{AssetMetadata, DailyPrice, Dividend, MacroObservation, MacroSeriesId, Split, TickerSearchHit};

#[async_trait]
pub trait DataProvider: Send + Sync {
    fn name(&self) -> &'static str;

    async fn search_tickers(&self, query: &str, limit: u32) -> ProviderResult<Vec<TickerSearchHit>>;
    async fn fetch_metadata(&self, ticker: &str) -> ProviderResult<AssetMetadata>;
    async fn fetch_prices(&self, ticker: &str, start: NaiveDate, end: NaiveDate) -> ProviderResult<Vec<DailyPrice>>;
    async fn fetch_dividends(&self, ticker: &str, start: NaiveDate, end: NaiveDate) -> ProviderResult<Vec<Dividend>>;
    async fn fetch_splits(&self, ticker: &str, start: NaiveDate, end: NaiveDate) -> ProviderResult<Vec<Split>>;
}

#[async_trait]
pub trait MacroProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn fetch_series(&self, series: MacroSeriesId, start: NaiveDate, end: NaiveDate) -> ProviderResult<Vec<MacroObservation>>;
}
```

### `MacroSeriesId`

Massive's `/fed/v1/*` endpoints return *bundles* of related indicators per row (e.g. `/fed/v1/treasury-yields` returns yields at every standard tenor for each date). The adapter explodes one API row into N `MacroObservation`s, one per series_id (e.g. `treasury_3mo`, `treasury_10y`, `cpi_yoy`, `cpi_core_yoy`, `unemployment_rate`). Defined as a closed enum in `sync/types.rs`:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MacroSeriesId {
    Treasury3Mo,
    Treasury2Y,
    Treasury10Y,
    Treasury30Y,
    CpiYoy,
    CpiCoreYoy,
    UnemploymentRate,
    FedFundsRate,
    // extend as needed
}
```

The serialized lower-snake form is what gets stored as `macro_series.series_id`. Phase 1 covers at minimum `Treasury3Mo`, `Treasury10Y`, `CpiYoy`, `UnemploymentRate`.

### Error model

```rust
#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("missing api key for provider {0}")]
    MissingApiKey(&'static str),

    #[error("rate limited by provider {0} — retry after {1:?}")]
    RateLimited(&'static str, Option<std::time::Duration>),

    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("provider returned status {status} for {url}: {body}")]
    BadStatus { status: u16, url: String, body: String },

    #[error("response decode error: {0}")]
    Decode(String),

    #[error("ticker not found: {0}")]
    NotFound(String),

    #[error("free-tier limit reached: {0}")]
    FreeTierLimit(&'static str),
}
```

### Rate limit config

- **Massive free tier:** `Quota::per_minute(NonZeroU32::new(5).unwrap())` with no burst. The orchestrator must call `until_ready()` before every request and treat a 429 as `RateLimited` with `Retry-After` honored.
- **Massive paid tier (Stocks Starter and above):** `Quota::per_second(NonZeroU32::new(50).unwrap())` with burst 100 — selected at startup based on a `MASSIVE_TIER` env var (defaults to `free`).

### Verification (1A)

- [ ] `cargo check -p chrdfin-desktop` passes.
- [ ] `cargo clippy -p chrdfin-desktop -- -D warnings` clean.
- [ ] `cargo test -p chrdfin-desktop sync::rate_limit` covers: 6th request inside a 1-minute window blocks until the window rolls.
- [ ] App launches with no `MASSIVE_API_KEY` set (logs a warning, no panic).

### Commit message draft (1A)

```
feat(sync): add provider trait, http client, secrets, rate limiter

Lays the foundation for the Phase 1 data layer. Adds a reusable
reqwest-based AppHttpClient on AppState, a startup-time secrets
loader for MASSIVE_API_KEY, the DataProvider/MacroProvider traits
that every adapter will implement, and a governor-backed rate
limiter pre-configured for Massive's free tier (5 RPM).

No adapter is wired in yet — that lands in 1B.
```

---

## Sub-phase 1B: Massive adapter

Implement `MassiveProvider` against every endpoint Phase 1 needs. All HTTP traffic is mockable via `wiremock`. No DB writes yet — pure provider work.

### Files to add

- `apps/desktop/src-tauri/src/sync/massive/mod.rs` — re-exports.
- `apps/desktop/src-tauri/src/sync/massive/client.rs` — `MassiveProvider` struct (`http`, `api_key`, `rate_limiter`, `base_url`).
- `apps/desktop/src-tauri/src/sync/massive/aggregates.rs` — `fetch_prices`, `fetch_previous_close`.
- `apps/desktop/src-tauri/src/sync/massive/reference.rs` — `search_tickers`, `fetch_metadata`.
- `apps/desktop/src-tauri/src/sync/massive/corporate.rs` — `fetch_dividends`, `fetch_splits`.
- `apps/desktop/src-tauri/src/sync/massive/macro_series.rs` — `fetch_treasury_yields`, `fetch_inflation`, plus the explode-to-`MacroObservation` mapping.
- `apps/desktop/src-tauri/src/sync/massive/market.rs` — `fetch_market_status`, `fetch_market_holidays`.
- `apps/desktop/src-tauri/src/sync/massive/limits.rs` — named constants for free-tier caveats (`FREE_TIER_RPM = 5`, `FREE_TIER_HISTORY_YEARS = 2`).

### Test fixtures

Captured live (with the user's free-tier key, run once and committed) under `apps/desktop/src-tauri/tests/fixtures/massive/`:
- `aggs_spy_2024-01-01_2024-01-10.json`
- `tickers_search_apple.json`
- `ticker_overview_aapl.json`
- `dividends_aapl_2023.json`
- `splits_aapl_all.json`
- `treasury_yields_2024-Q1.json`
- `inflation_2024-Q1.json`
- `market_status.json`
- Trimmed to the smallest representative payloads.

### `Cargo.toml` additions (dev-dependencies)

```toml
[dev-dependencies]
wiremock = "0.6"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

### Implementation notes

- Auth: `Authorization: Bearer <key>` header on every request.
- Aggregates response: `results` array with `t` (epoch ms), `o`, `h`, `l`, `c`, `v`, `vw`, `n`. Convert `t` → `NaiveDate` via `chrono::DateTime::from_timestamp_millis`. `adj_close` is `c` when `adjusted=true` (the default); pass `adjusted=true` always.
- Pagination: Massive's reference + dividends + splits + macro endpoints all paginate via `next_url`. `MassiveProvider` follows `next_url` until exhausted (or until a configurable cap to avoid runaway loops on broad queries).
- 404 from `/v3/reference/tickers/{ticker}` and from aggregates with no rows → `ProviderError::NotFound`.
- 429 → parse `Retry-After`, surface as `ProviderError::RateLimited`.
- Macro explode: `/fed/v1/treasury-yields` returns rows like `{ "date": "2024-01-02", "yield_3_month": 5.39, "yield_10_year": 3.94, ... }`. The adapter maps each populated tenor field to a separate `MacroObservation { series: Treasury3Mo, date, value }`. Same pattern for `/fed/v1/inflation`.

### Tests (unit, mocked HTTP)

Per-endpoint:
- `massive::aggregates — happy path 5 days SPY`
- `massive::aggregates — empty range returns empty Vec`
- `massive::aggregates — 404 maps to NotFound`
- `massive::aggregates — 429 maps to RateLimited with Retry-After`
- `massive::reference — search_tickers paginates next_url`
- `massive::reference — fetch_metadata happy path`
- `massive::corporate — dividends date range filter is inclusive`
- `massive::corporate — splits returns forward + reverse + stock dividend types`
- `massive::macro — treasury_yields explodes one API row into N observations`
- `massive::macro — inflation drops null fields gracefully`
- `massive::* — bearer header is set`

### Verification (1B)

- [ ] `cargo test -p chrdfin-desktop sync::massive` all green.
- [ ] No real network call required for the unit suite.
- [ ] One `#[ignore]`-tagged live integration test per category for opt-in human verification (`cargo test -- --ignored massive_live`). Never runs in CI.

### Commit message draft (1B)

```
feat(sync): implement Massive provider adapter

Adds MassiveProvider implementing DataProvider + MacroProvider
against api.massive.com. Covers daily aggregates, ticker reference
+ search, single-ticker overview, dividends, splits, treasury
yields, inflation, and market status. Macro endpoints are exploded
from per-date bundle rows into per-series MacroObservations.

All endpoints have wiremock-backed unit tests using committed
fixtures. Live integration tests are marked #[ignore] and opt-in.

No DB writes or commands yet — that lands in 1C and 1D.
```

---

## Sub-phase 1C: Storage layer + schema migration

Typed write/read helpers over DuckDB. Adds a `source` column to every persisted-data table so future providers (or a bulk historical backfill) can coexist with Massive-sourced rows.

### Schema migration (additive, idempotent)

Add to `apps/desktop/src-tauri/src/schema.sql`:

```sql
ALTER TABLE daily_prices ADD COLUMN IF NOT EXISTS source VARCHAR(20) DEFAULT 'massive';
ALTER TABLE dividends    ADD COLUMN IF NOT EXISTS source VARCHAR(20) DEFAULT 'massive';
ALTER TABLE macro_series ADD COLUMN IF NOT EXISTS source VARCHAR(20) DEFAULT 'massive';
ALTER TABLE assets       ADD COLUMN IF NOT EXISTS source VARCHAR(20) DEFAULT 'massive';

CREATE TABLE IF NOT EXISTS splits (
    ticker          VARCHAR(20) NOT NULL REFERENCES assets(ticker),
    execution_date  DATE NOT NULL,
    split_from      DOUBLE NOT NULL,    -- e.g. 1.0 in a 4-for-1
    split_to        DOUBLE NOT NULL,    -- e.g. 4.0 in a 4-for-1
    adjustment_type VARCHAR(20),         -- 'forward_split' | 'reverse_split' | 'stock_dividend'
    source          VARCHAR(20) DEFAULT 'massive',
    PRIMARY KEY (ticker, execution_date)
);
```

> The `IF NOT EXISTS` clauses make this safe to re-run on every app launch. DuckDB applies `ALTER TABLE … ADD COLUMN IF NOT EXISTS` as a no-op when the column exists.
> Update `docs/database-schema-reference.md` to reflect the new `source` column and the new `splits` table.

### Source-priority constant

```rust
// apps/desktop/src-tauri/src/storage/source.rs

/// Higher number = higher priority. A row from a higher-priority source
/// is never overwritten by a lower-priority source on conflict. Phase 1
/// has only one provider, so this is forward-looking infrastructure.
pub const SOURCE_PRIORITY: &[(&str, u8)] = &[
    ("manual_backfill", 100),  // bulk historical Parquet imports beat everything
    ("massive", 50),
];

pub fn priority(source: &str) -> u8 {
    SOURCE_PRIORITY
        .iter()
        .find(|(name, _)| *name == source)
        .map(|(_, prio)| *prio)
        .unwrap_or(0)
}
```

Upserts compare priorities and skip the write if the existing row's source ranks higher. SQL idiom:

```sql
INSERT INTO daily_prices (ticker, date, open, high, low, close, adj_close, volume, source)
VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
ON CONFLICT (ticker, date) DO UPDATE SET
    open = EXCLUDED.open,
    high = EXCLUDED.high,
    low = EXCLUDED.low,
    close = EXCLUDED.close,
    adj_close = EXCLUDED.adj_close,
    volume = EXCLUDED.volume,
    source = EXCLUDED.source
WHERE source_priority(EXCLUDED.source) >= source_priority(daily_prices.source);
```

`source_priority` is registered as a DuckDB scalar UDF at connection-init time.

### Files to add

- `apps/desktop/src-tauri/src/storage/mod.rs` — module root + UDF registration.
- `apps/desktop/src-tauri/src/storage/source.rs` — priority table + UDF.
- `apps/desktop/src-tauri/src/storage/assets.rs` — `upsert_asset`, `get_asset`, `search_assets_local`.
- `apps/desktop/src-tauri/src/storage/prices.rs` — `upsert_prices` (batch via DuckDB appender where possible), `get_prices`, `latest_price_date`.
- `apps/desktop/src-tauri/src/storage/dividends.rs` — `upsert_dividends`, `get_dividends`.
- `apps/desktop/src-tauri/src/storage/splits.rs` — `upsert_splits`, `get_splits`.
- `apps/desktop/src-tauri/src/storage/macros.rs` — `upsert_macro_observations`, `get_macro_series`, `latest_macro_date`.
- `apps/desktop/src-tauri/src/storage/sync_log.rs` — `start_sync`, `complete_sync`, `fail_sync`, `last_successful_sync`.
- `apps/desktop/src-tauri/src/storage/settings.rs` — `get_setting<T>`, `set_setting<T>` over the `app_settings` JSON column.

### Tests (integration, in-memory DuckDB)

`apps/desktop/src-tauri/tests/storage.rs`:
- `upsert_prices — inserts new rows`
- `upsert_prices — updates existing rows on conflict when same source`
- `upsert_prices — does NOT overwrite higher-priority source row`
- `latest_price_date — None when ticker absent, max date otherwise`
- `upsert_macro_observations — handles empty input and dedupes by (series, date)`
- `sync_log — start/complete pair persists status and aggregates`
- `app_settings — set and get round-trip a JSON value`
- `splits — upsert + read round-trip`

Use `Connection::open_in_memory()` and execute the same `schema.sql` per test to bootstrap.

### Verification (1C)

- [ ] `cargo test -p chrdfin-desktop storage` all green.
- [ ] Existing `chrdfin.duckdb` files in development still open and accept the new `ALTER TABLE` migration without data loss.

### Commit message draft (1C)

```
feat(storage): add source-aware upsert layer + splits table

Introduces apps/desktop/src-tauri/src/storage with typed upsert
and read helpers for assets, daily_prices, dividends, splits,
macro_series, sync_log, and app_settings. Adds a `source` column
to every persisted-data table and a SOURCE_PRIORITY table that
ensures higher-priority sources (e.g. manual bulk backfills)
are never overwritten by lower-priority API fetches.

The schema migration is additive and idempotent — existing
chrdfin.duckdb files migrate cleanly on next app launch.
```

---

## Sub-phase 1D: Sync orchestrator + `sync_data` command

Tie providers and storage together. This is the only place that decides what to fetch when.

### Files to add

- `apps/desktop/src-tauri/src/sync/orchestrator.rs` — `Sync::run`, `Sync::ensure_ticker`.
- `apps/desktop/src-tauri/src/commands/sync.rs` — `sync_data`, `get_sync_status`.
- `apps/desktop/src-tauri/src/commands/mod.rs` — register the new module.
- `apps/desktop/src-tauri/src/lib.rs` — register both commands in `generate_handler!`.

### Orchestrator surface

```rust
pub struct Sync {
    pub massive: Arc<MassiveProvider>,
    pub db: Arc<Database>,
    pub running: Arc<tokio::sync::Mutex<()>>,  // gates manual + scheduled overlap
    pub in_flight: DashMap<String, Arc<tokio::sync::Mutex<()>>>,  // per-ticker dedup
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncMode {
    Full,        // back to ticker's first_date or 2 years (free-tier history cap)
    Incremental, // from latest stored date forward
}

#[derive(Debug, Clone, Serialize)]
pub struct SyncProgress {
    pub phase: String,           // "metadata" | "prices" | "dividends" | "splits" | "macro"
    pub current: u32,
    pub total: u32,
    pub message: Option<String>,
}

impl Sync {
    pub async fn run(
        &self,
        mode: SyncMode,
        on_progress: impl Fn(SyncProgress) + Send + Sync + 'static,
    ) -> AppResult<SyncSummary> { /* ... */ }

    pub async fn ensure_ticker(
        &self,
        ticker: &str,
        start: NaiveDate,
        end: NaiveDate,
    ) -> AppResult<()> { /* ... */ }
}
```

### Behavior

- **Universe:** read `tracked_universe` (JSON array) from `app_settings`. The starter universe is seeded in 1F.
- **Macro series:** fixed list — `Treasury3Mo`, `Treasury10Y`, `CpiYoy`, `UnemploymentRate`.
- **Concurrency:** **1 in-flight request at a time** while on the free tier (5 RPM cap is the binding constraint, parallelism would only fight the limiter). Configurable via `MASSIVE_TIER`.
- **Incremental:** for each ticker compute `start = latest_price_date + 1 day`, `end = today`. Skip ticker if already up to date.
- **Full:** `start = max(asset.first_date, today - 2 years)`. The 2-year clamp matches the free-tier cap; remove when an upgraded plan is detected.
- **Free-tier resilience:** if a 429 burst happens, sleep `Retry-After` seconds and resume from the same ticker. If a hard `FreeTierLimit` is hit (e.g. monthly cap), finish what's possible, write the failure to `sync_log`, and surface it in `SyncSummary.errors` as a warning — do not abort.
- **Per-ticker error isolation:** one failed ticker never aborts the run.
- **Logging:** one `sync_log` row per `Sync::run` invocation with aggregated `tickers_synced` and `rows_upserted`.

### `sync_data` command

```rust
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncDataInput {
    pub mode: SyncMode,
    pub include_macro: Option<bool>, // default true
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SyncSummary {
    pub started_at: String,
    pub completed_at: String,
    pub mode: SyncMode,
    pub tickers_synced: u32,
    pub rows_upserted: u32,
    pub errors: Vec<SyncError>,
}

#[tauri::command]
pub async fn sync_data(
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
    input: SyncDataInput,
) -> Result<SyncSummary, String> {
    // emits "sync:progress" repeatedly, "sync:completed" or "sync:failed" once.
}

#[tauri::command]
pub fn get_sync_status(state: tauri::State<'_, AppState>) -> Result<SyncStatus, String> {
    // reads latest sync_log row + last_successful_sync timestamp + currently-running flag.
}
```

### Tauri events

- `sync:progress` → `SyncProgress`
- `sync:completed` → `SyncSummary`
- `sync:failed` → `{ error: string }`

### Tests

- `Sync::run(Full)` against mocked Massive + in-memory DB — populates `daily_prices`, `dividends`, `splits`, `macro_series`.
- `Sync::run(Incremental)` recognizes existing data and only fetches the gap.
- `ensure_ticker` deduplicates concurrent calls for the same ticker (via the per-ticker mutex).
- 429 mid-run is retried after `Retry-After` and the run completes.
- `FreeTierLimit` mid-run finalizes with `errors` populated and `completed_at` set.

### Verification (1D)

- [ ] `cargo test -p chrdfin-desktop sync::orchestrator` all green.
- [ ] `cargo check -p chrdfin-desktop` passes with new commands registered.
- [ ] Manual: `pnpm tauri dev`, invoke `sync_data` from the dev console with `mode: "incremental"` against an empty DB, observe `sync:progress` events and a final `sync:completed`.

### Commit message draft (1D)

```
feat(sync): add orchestrator, sync_data command, and progress events

Wires the Massive provider and the storage layer together behind
a Sync orchestrator that supports Full and Incremental modes.
Exposes sync_data and get_sync_status Tauri commands. Long-running
syncs emit sync:progress events and finalize with sync:completed
or sync:failed.

Per-ticker errors are isolated, the free-tier 5 RPM rate limit is
respected, and 429 responses are retried after Retry-After seconds.
A run-level mutex prevents manual + scheduled syncs from overlapping.

ensure_ticker is the on-demand entry point used by read commands
in 1E to fetch unknown tickers transparently.
```

---

## Sub-phase 1E: Read commands + frontend wiring

Pure DuckDB readers exposed as Tauri commands, plus the TanStack Query layer and a minimal SyncStatus UI in the platform shell.

### Files to add

- `apps/desktop/src-tauri/src/commands/data.rs` — `get_prices`, `get_macro_series`, `get_asset_metadata`, `search_tickers`.
- `apps/desktop/src/lib/queries/sync.ts` — `useSyncStatus`, `useSyncDataMutation`, `useSyncProgress`.
- `apps/desktop/src/lib/queries/market-data.ts` — `usePrices`, `useTickerSearch`, `useAssetMetadata`, `useMacroSeries`.
- `apps/desktop/src/lib/queryKeys.ts` — central key factory.
- `apps/desktop/src/components/shell/SyncStatusBadge.tsx` — header indicator (idle / syncing / failed / up-to-date).
- `apps/desktop/src/components/dev/DataLayerCard.tsx` — temporary dev card on the Dashboard placeholder. Removed in Phase 10 when proper Settings ships.
- `apps/desktop/src/routes/__root.tsx` — render `<SyncStatusBadge />` in the header right slot.
- `apps/desktop/src/routes/index.tsx` — render `<DataLayerCard />`.

### Read command surface

```rust
#[tauri::command]
pub async fn get_prices(
    state: tauri::State<'_, AppState>,
    ticker: String,
    start: String,
    end: String,
) -> Result<Vec<DailyPrice>, String> {
    // 1. If asset absent or coverage incomplete, call Sync::ensure_ticker.
    // 2. Read from storage::prices::get_prices.
}

#[tauri::command]
pub fn get_macro_series(
    state: tauri::State<'_, AppState>,
    series_id: String,   // serialized MacroSeriesId, e.g. "treasury_10_y"
    start: String,
    end: String,
) -> Result<Vec<MacroObservation>, String> { /* storage::macros */ }

#[tauri::command]
pub async fn get_asset_metadata(
    state: tauri::State<'_, AppState>,
    ticker: String,
) -> Result<AssetMetadata, String> {
    // Local-first; ensure_ticker if missing.
}

#[tauri::command]
pub async fn search_tickers(
    state: tauri::State<'_, AppState>,
    query: String,
    limit: Option<u32>,
) -> Result<Vec<TickerSearchHit>, String> {
    // Local-first via storage::assets::search_assets_local.
    // If results < limit, fall through to Massive search and merge unique results.
}
```

### TypeScript types

Verify the read-side mirrors exist in `packages/types/src/market-data.ts` (`DailyPrice`, `MacroObservation`, `AssetMetadata`, `TickerSearchHit`); add if missing per `docs/type-definitions-reference.md`. Re-export from `packages/types/src/index.ts`.

### Query-key factory

```typescript
export const qk = {
  syncStatus: () => ["sync", "status"] as const,
  prices: (ticker: string, range: { start: string; end: string }) =>
    ["prices", ticker, range] as const,
  asset: (ticker: string) => ["asset", ticker] as const,
  search: (query: string) => ["search", query] as const,
  macro: (series: string, range: { start: string; end: string }) =>
    ["macro", series, range] as const,
};
```

### Stale times

- Prices: 5 min
- Macro: 1 hour
- Asset metadata: 1 hour
- Sync status: 0 (always refetch on focus)

### Invalidation

After `sync:completed`, invalidate `["prices"]`, `["macro"]`, `["asset"]` query roots so any visible chart or detail panel refetches against fresh data.

### `<SyncStatusBadge />` states

| State | Visual |
|---|---|
| Up to date | Green dot, "Synced 2m ago" |
| Syncing | Spinner, "Syncing prices… 12/26" |
| Failed | Amber dot, "Last sync failed — click to retry" |
| Never run | Gray dot, "No sync yet — click to seed" |

### `<DataLayerCard />` (Dashboard placeholder)

A single Card with:
- Sync status row + "Run Incremental" + "Run Full" buttons.
- Live progress bar bound to `useSyncProgress`.
- Recent `sync_log` rows table (last 10 entries).
- Clearly labeled "Developer view — replaced by Settings in Phase 10".

### Tests

- `cargo test -p chrdfin-desktop commands_data` covers `get_prices` + on-demand fetch, date filter inclusivity, search local-then-remote, macro happy path.
- `pnpm test --filter desktop` for `queryKeys` factory + a smoke test that `useSyncProgress` updates in response to a fake `sync:progress` event.

### Verification (1E)

- [ ] `pnpm typecheck`, `pnpm lint`, `pnpm format:check` all clean.
- [ ] `cargo test -p chrdfin-desktop commands_data` green.
- [ ] Manual: launch app, click "Run Incremental" in the dev card with starter universe seeded, watch SyncStatusBadge spin, see progress in the card, verify `sync_log` table updates.
- [ ] Manual: with a populated DB, call `get_prices('SPY', '2024-01-01', '2024-12-31')` from the dev console — receives a non-empty array.
- [ ] Manual: call `get_prices('NEWLY_REQUESTED_TICKER', ...)` — observe an on-demand fetch, then rows.

### Commit message draft (1E)

```
feat(data): add read commands, query hooks, and SyncStatusBadge

Exposes get_prices, get_macro_series, get_asset_metadata, and
search_tickers as Tauri commands. Read commands are local-first
against DuckDB, falling through to Sync::ensure_ticker when the
local store doesn't cover the requested range.

Adds the @chrdfin TanStack Query layer (qk factory, hooks for
each read command, sync mutation hooks, real-time progress
subscription) and a SyncStatusBadge in the header. A temporary
DataLayerCard on the dashboard exposes Run Incremental / Run Full
buttons for development; it gets replaced by proper Settings UI
in Phase 10.
```

---

## Sub-phase 1F: Background scheduler + seed + docs

Final sub-phase. Adds the background sync task, the first-launch seed flow, and the documentation updates that land alongside Phase 1.

### Background scheduler

- `apps/desktop/src-tauri/src/sync/scheduler.rs` — `spawn_background_sync(state: Arc<AppState>) -> JoinHandle<()>`.
- `apps/desktop/src-tauri/Cargo.toml`: add `chrono-tz = "0.10"`.

Behavior:
- **On launch:** `Sync::seed_if_needed` first (see below). Then if `last_successful_sync` is `None` or older than 24 hours, kick off `Sync::run(Incremental)`.
- **Daily loop:** sleep until the next 6:00 PM America/New_York (DST-aware via `chrono-tz`), then run `Sync::run(Incremental)`. Loop forever.
- **Weekend skip:** if next 6 PM ET falls on Saturday or Sunday, sleep to the next weekday's 6 PM instead.
- **Failure backoff:** failed run → log + retry in 1 hour, capped at 4 retries before falling back to next-day cadence.
- **Overlap-safe:** uses the same `Sync::running` mutex as manual `sync_data`.

Tests:
- `scheduler::next_run_at` — Friday → Monday, Saturday → Monday, Sunday → Monday, weekday before/after 6 PM, DST spring-forward, DST fall-back.

### Starter universe + seed

`packages/config/src/constants.ts`:

```typescript
export const STARTER_UNIVERSE = [
  // Core ETFs
  "SPY", "QQQ", "IWM", "DIA", "VTI", "VOO", "VEA", "VWO",
  // Bonds
  "AGG", "BND", "TLT", "IEF", "SHY", "TIP",
  // Sector + Commodities
  "GLD", "SLV", "USO", "VNQ",
  // Major single names
  "AAPL", "MSFT", "NVDA", "AMZN", "GOOGL", "META", "TSLA", "BRK-B",
] as const satisfies readonly string[];

export const DEFAULT_MACRO_SERIES = [
  "treasury_3_mo", "treasury_10_y", "cpi_yoy", "unemployment_rate",
] as const satisfies readonly string[];
```

> 26 tickers × ~2 years EOD ≈ ~13,000 rows. At 5 RPM the first full seed completes in ~6 minutes — acceptable for first-launch UX with progress feedback. Macro adds 4 more requests on top.

`Sync::seed_if_needed`:
1. Read `app_settings.seeded` — if `true`, return.
2. Set `app_settings.tracked_universe` from `STARTER_UNIVERSE`.
3. Run `Sync::run(Full)`.
4. Flip `app_settings.seeded = true` once at least one ticker succeeds (so partial completion still moves forward; subsequent runs incrementally top up).

### Capabilities

- `apps/desktop/src-tauri/capabilities/default.json` — verify `core:event:default` is granted (frontend listens to `sync:*` events). No per-command capability needed — `generate_handler!` auto-discovers.

### CI

`.github/workflows/ci.yml` — add to the `rust-test` job (push-to-main only):

```yaml
- name: cargo test (chrdfin-desktop, mocked + in-memory)
  run: cargo test -p chrdfin-desktop
```

PR feedback stays fast because the `rust-fast` job still only runs `cargo check` + `cargo clippy`.

### Documentation updates

- **`CLAUDE.md`** — append to "Common Gotchas":
  - Massive (Polygon rebrand) free tier is 5 RPM and ~2 years history. Configure `MASSIVE_TIER=free` to enable conservative limits; default to free until proven upgraded.
  - Massive's `/fed/v1/*` macro endpoints return *bundles* per row; the adapter explodes them into per-`MacroSeriesId` observations.
  - Splits adjustment: Massive returns `split_from`/`split_to` ratios. Backtest math should multiply historical share counts by `split_to / split_for_split_from`.
  - All HTTP egress goes through `AppHttpClient` in the Rust backend; never `fetch()` from the webview (avoids CORS, prevents key leakage to JS).
- **`docs/agent-handoff.md`** — add `docs/sync-architecture.md` to the inventory and to the "Sync, providers, and the data layer" task row.
- **New: `docs/sync-architecture.md`** — short doc covering: provider trait + adapter pattern, source-priority model, orchestrator + scheduler lifecycle, on-demand fetch dedup, free-tier vs paid-tier configuration, where to add a second adapter (Schwab/Tradier hooks for the post-1.0 trading roadmap). Links from `docs/agent-handoff.md`.
- **`docs/database-schema-reference.md`** — document the new `source` column on the four affected tables and the new `splits` table.
- **`docs/data-fetching-patterns.md`** — register the `qk` keys (`prices`, `asset`, `search`, `macro`, `syncStatus`) and the `sync:completed → invalidate(prices, asset, macro)` graph.
- **`docs/technical-blueprint.md`** — update the Phase 1 section: deliverables collapse to "Massive sole provider, source-aware schema, orchestrator + scheduler, read commands, seed". Strike Tiingo and FRED references.
- **`CHANGELOG.md`** — populate `[Unreleased]` with the Phase 1 highlights.
- **`.env.example`** — add `MASSIVE_API_KEY=` and `MASSIVE_TIER=free` (defaults). Remove the Tiingo / FRED / Polygon entries that no longer apply.

### Verification (1F)

- [ ] Fresh `chrdfin.duckdb` (delete the file): launch app, observe the seed running, `daily_prices` populates for the starter universe, `app_settings.seeded` flips to `true`.
- [ ] Relaunch with a stale `last_successful_sync`: scheduler kicks an automatic incremental sync.
- [ ] `cargo test -p chrdfin-desktop scheduler::next_run_at` covers the weekday/weekend/DST cases.
- [ ] CI: PR + main push both pass under the new test wiring.

### Commit message draft (1F)

```
feat(sync): add background scheduler, first-launch seed, and docs

Spawns a background tokio task on launch that runs Sync::seed_if_needed
followed by an incremental sync if the local store is stale (>24h).
Schedules a daily incremental at 6 PM America/New_York with weekend
skip and DST-aware scheduling via chrono-tz. Manual and scheduled
runs share a single mutex so they never overlap.

Defines the STARTER_UNIVERSE (26 tickers) and DEFAULT_MACRO_SERIES
(4 series) in @chrdfin/config. The seed populates app_settings.
tracked_universe and runs Sync::run(Full); partial completion is
acceptable — subsequent runs incrementally top up.

Adds docs/sync-architecture.md, updates database-schema-reference,
data-fetching-patterns, technical-blueprint Phase 1 section, and
CLAUDE.md gotchas. CI now runs cargo test -p chrdfin-desktop on
push to main.
```

---

## Pull Request — open after 1F lands

A single PR titled **`feat: Phase 1 — Massive-backed data layer with background sync`**. Body:

```markdown
## Summary
- Massive (Polygon rebrand) is the sole equity + macro data provider for Phase 1.
- Adds a source-aware DuckDB storage layer + splits table; existing chrdfin.duckdb files migrate cleanly.
- Implements Sync orchestrator with manual + scheduled execution, free-tier rate limiting, on-demand fetch, and progress events.
- Adds get_prices / get_macro_series / get_asset_metadata / search_tickers Tauri commands and the matching TanStack Query hooks.
- SyncStatusBadge in the header + temporary DataLayerCard on the dashboard for dev-driven syncs.
- First-launch seed populates a 26-ticker starter universe; daily 6 PM ET incremental sync runs in the background.

## Test plan
- [ ] Fresh DB launches, seeds, populates daily_prices for starter universe
- [ ] sync_data emits sync:progress events and finalizes with sync:completed
- [ ] On-demand fetch: get_prices for an unknown ticker triggers ensure_ticker and returns rows
- [ ] Free-tier 429 mid-run retries after Retry-After and the run completes
- [ ] Scheduler runs incremental sync when last_successful_sync is stale
- [ ] cargo test --workspace passes
- [ ] pnpm typecheck && pnpm lint && pnpm test all clean
```

Tag `v0.1.0` after the PR lands on `main`, per the established release policy in `.claude/instructions/changelog-and-releases.md`.

---

## Completion Checklist (full Phase 1)

Run these in order. All must pass before opening the PR:

```bash
pnpm install                              # No errors
pnpm typecheck                            # All packages pass
pnpm lint                                 # Zero warnings
pnpm test                                 # All TS tests pass
cargo check --workspace                   # Rust compiles
cargo clippy --workspace -- -D warnings   # No clippy warnings
cargo test --workspace                    # Rust tests pass (incl. mocked + in-memory)
pnpm tauri dev                            # App launches, sync runs end to end
```

Additionally verify:

- [ ] `MassiveProvider` implements `DataProvider` + `MacroProvider` and is unit-tested with mocked HTTP for every endpoint.
- [ ] `Sync::run(Full)` and `Sync::run(Incremental)` populate the DB correctly against a mocked provider.
- [ ] `sync_data` Tauri command emits `sync:progress` events and returns a `SyncSummary`.
- [ ] `get_sync_status` returns the latest `sync_log` snapshot.
- [ ] `get_prices`, `get_macro_series`, `get_asset_metadata`, `search_tickers` all return data from DuckDB.
- [ ] Unknown-ticker `get_prices` triggers an on-demand fetch and returns rows.
- [ ] Background scheduler runs an incremental sync on launch when stale and at the next 6 PM ET on a weekday.
- [ ] First-launch seed populates the starter universe and flips `app_settings.seeded`.
- [ ] Header `<SyncStatusBadge />` reflects current sync state.
- [ ] Dashboard `<DataLayerCard />` can trigger Full and Incremental sync and shows recent `sync_log` rows.
- [ ] Empty `.env` does not crash the app; a sync attempt surfaces `MissingApiKey` cleanly.
- [ ] Free-tier 429 is handled gracefully and the run resumes.
- [ ] `source` column persists `'massive'` for API-fetched rows; `source_priority` UDF prevents downgrades.
- [ ] CI passes both `rust-fast` (PR) and `rust-test` (main) jobs.

When all pass, Phase 1 is complete. The PR lands, the user tags `v0.1.0`, and Phase 2 (Computation Core) begins.
