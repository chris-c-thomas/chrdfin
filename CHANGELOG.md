# Changelog

All notable changes to chrdfin are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

While the project is pre-1.0, the minor version tracks the development phase
(`v0.N.0` = Phase N completion). Patch releases (`v0.N.M`) bundle bug fixes
within a phase. See `.claude/instructions/changelog-and-releases.md` for the
full versioning and release policy.

## [Unreleased]

### Added

- **Phase 1 — Data layer.** Massive (the rebrand of Polygon.io) is the sole
  equity + macro data provider for the Phase 1 ship. The `DataProvider`
  and `MacroProvider` traits leave room for additional adapters
  (Tiingo, Schwab, Tradier) at the source-priority level without
  restructuring storage or the orchestrator.
- **`Sync` orchestrator.** Drives full and incremental syncs with a
  run-level mutex, per-ticker dedup map (`DashMap` of async mutexes),
  one transparent retry on 429 (honors `Retry-After`), and per-ticker
  error isolation. Logs every run to `sync_log`.
- **On-demand fetch.** `Sync::ensure_ticker` lets `get_prices` and
  `get_asset_metadata` populate unknown tickers transparently —
  concurrent requests for the same symbol queue behind a single fetch.
- **Background scheduler.** `spawn_background_sync` runs
  `Sync::seed_if_needed` on launch, kicks an incremental sync if the
  local store is stale (>24 h), and then loops daily at **6 PM
  America/New_York** on weekdays (DST-aware via `chrono-tz`). Failed
  runs back off 1 h, capped at 4 retries before falling back to
  next-day cadence.
- **First-launch seed.** `STARTER_UNIVERSE` (26 ETFs + bonds +
  commodities + major single names) and `DEFAULT_MACRO_SERIES` (4
  series) populate on first launch, with `app_settings.seeded` tracking
  completion.
- **Read commands.** `get_prices`, `get_macro_series`,
  `get_asset_metadata`, and `search_tickers` Tauri commands.
  Local-first against DuckDB; on-demand fetch for unknown tickers;
  hybrid local + remote merge for search.
- **Source-aware schema.** New `source` column on `assets`,
  `daily_prices`, `dividends`, `splits`, and `macro_series`. New
  `splits` table. Source-priority model
  (`storage::source::SOURCE_PRIORITY`) prevents a lower-priority row
  from overwriting a higher-priority one.
- **Frontend wiring.** `qk` query-key factory, TanStack Query hooks for
  every read command + sync mutation + progress event,
  `<SyncStatusBadge />` in the header, and `<DataLayerCard />`
  developer surface on the dashboard (replaced by Settings UI in
  Phase 10).
- **`docs/sync-architecture.md`** — provider trait + adapter pattern,
  source-priority model, orchestrator + scheduler lifecycle, on-demand
  fetch dedup, free-tier vs paid-tier configuration, where to add a
  second adapter, storage-backend swap path.

### Changed

- **CI restructured for capacity and feedback latency.** Split the Rust
  job into a fast path (`cargo check` + `cargo clippy` per crate,
  blocks PR merges) and a slow path (`cargo test`, runs only on push
  to `main`). Added a "Free disk space" step that reclaims ~25–30 GB
  of preinstalled runner tooling so the bundled-DuckDB build stops
  exhausting disk. Tightened the `Swatinem/rust-cache@v2` cache with
  `shared-key` and `save-if: refs/heads/main` so PRs read the cache
  but don't pollute it, and unchanged crates survive `Cargo.lock`
  drift. The `rust-test` job now runs `cargo test -p chrdfin-desktop`
  (mocked HTTP + in-memory DuckDB; no real network) on push to `main`.
- **Removed `staticlib` from the Tauri desktop crate's
  `[lib].crate-type`.** Tauri v2 desktop builds only need `cdylib`
  (Tauri loads at runtime) and `rlib` (for `main.rs` to link). The
  `staticlib` archive was the largest single artifact in the build and
  wasn't consumed anywhere.
- **Provider DTOs serialize as camelCase** over Tauri IPC so the
  TypeScript layer round-trips cleanly without ad-hoc renaming.

### Removed

- **Tiingo and FRED references.** Phase 1 consolidated on Massive as
  the sole provider for both equity and macro data; the Tiingo and
  FRED adapter sketches that were called out earlier never shipped and
  are dropped from the architecture.

## [0.0.1] - 2026-04-25

Phase 0 — Foundation & Tooling. The first tagged build of chrdfin: a running
Tauri v2 desktop application with the platform shell rendered, the DuckDB
schema initialized, the Rust computation crate linked, and an end-to-end
IPC round-trip verified. No user-facing features yet — every domain renders
a "Coming in Phase N" placeholder. Shipped as a developer build only; not
distributed.

### Added

- **Monorepo scaffold.** Turborepo + pnpm workspaces with seven internal
  packages under the `@chrdfin/*` scope: `tsconfig`, `eslint-config`, `types`,
  `config`, `ui`, `charts`, plus the `desktop` app at `apps/desktop`.
- **Cargo workspace.** Edition 2024 with resolver 3, pinned to Rust 1.95+ via
  `rust-toolchain.toml`. Members: `crates/chrdfin-core` (computation engine,
  module stubs only) and `apps/desktop/src-tauri` (Tauri backend).
- **Tauri v2 desktop application.** Window 1440×900, IBM Plex Sans/Mono
  bundled locally via `@fontsource`, dark theme by default, Carbon-derived
  semantic color tokens, and a 56px-tall platform shell with sidebar and
  header.
- **Platform shell.** Collapsible sidebar (240px ⇄ 48px) with feature-flag
  gated nav items, top header with breadcrumbs, command-palette trigger,
  market-status indicator (NYSE hours via Intl + 2026 holiday calendar),
  and a theme toggle.
- **Sidebar navigation.** Top-level Dashboard entry plus four sections —
  Tracking, Analysis & Tools, Market, Reference — with plural labels
  (`Portfolios`, `Watchlists`, `Screeners`, `Calendars`) signalling
  multi-instance domains.
- **Dashboard placeholder.** Home screen at `/` listing the planned widget
  set (Portfolio Summary, Market Overview, Recent Backtests, Accounts, News,
  Earnings & Calendar) with the Phase 0 IPC health-check rendered as a
  subordinate card.
- **TanStack Router integration.** File-based routing with placeholder
  routes for all 23 surfaces across Tracking, Analysis, Tools, Market, and
  Reference. Section-level `beforeLoad` redirects driven by feature flags;
  route search params validated via Zod schemas in `@chrdfin/types`.
- **DuckDB embedded database.** 14-table schema applied idempotently on app
  launch from `schema.sql`: `assets`, `daily_prices`, `dividends`,
  `macro_series`, `portfolios`, `simulation_results`, `holdings`,
  `transactions`, `watchlists`, `news_articles`, `earnings_calendar`,
  `saved_calculator_states`, `app_settings`, `sync_log`. Database file
  located via Tauri's `app_data_dir()`.
- **Tauri commands.** `health_check` (DB liveness + crate version round-trip),
  `get_theme`, `set_theme` (theme preference persisted in the `app_settings`
  table; no localStorage).
- **Shared frontend infrastructure.** `useTauriQuery` and `useTauriMutation`
  TanStack Query wrappers, `ThemeProvider` backed by Tauri commands,
  `useMarketStatus` hook, `<DeltaValue>` and shadcn-derived primitives in
  `@chrdfin/ui` (Button, Card, Sidebar, Breadcrumb, Command, Dialog, Tooltip,
  Separator).
- **Feature flags.** Thirteen flags in `@chrdfin/config` covering every
  domain: `backtest`, `monteCarlo`, `tracker`, `optimizer`,
  `allocationOptimizer`, `calculators`, `marketData`, `news`, `research`,
  `reference`, plus the post-v1.0 trading reservations `paperTrading`,
  `liveTrading`, `botTrading`.
- **`PortfolioType` taxonomy.** Five classifications — `tracked`, `backtest`,
  `model`, `watchlist`, `paper` — with `paper` reserved for the post-v1.0
  paper-trading roadmap.
- **GitHub Actions CI.** Two-job workflow on push and PR: TypeScript Quality
  (typecheck → lint → format:check → test) and Rust Quality
  (`cargo fmt --check` → `cargo check` → `cargo clippy -D warnings` →
  `cargo test`). Concurrency-grouped per branch.
- **Vitest** workspace configuration with 24 placeholder tests across
  `@chrdfin/types`, `@chrdfin/config`, and `@chrdfin/ui`.
- **Documentation.** Eleven canonical specs in `docs/`:
  `agent-handoff.md`, `technical-blueprint.md`, `phase-0-checklist.md`,
  `database-schema-reference.md`, `type-definitions-reference.md`,
  `ui-design-system.md`, `ui-component-recipes.md`, `chart-recipes.md`,
  `route-conventions.md`, `data-fetching-patterns.md`, `form-patterns.md`.
  Plus `CLAUDE.md` (project conventions) and instruction documents in
  `.claude/instructions/` covering coding standards, code review, and
  changelog/release policy.

### Changed

- **Type scale aligned with Tailwind v4 defaults.** Removed a
  `html { font-size: 13px }` override that had silently scaled every
  rem-based utility down to ~81%. Migrated `text-md` references (a
  non-default utility) to standard `text-base` / `text-lg`. The webview
  now respects user-agent and OS accessibility scaling.
- **Sidebar geometry.** Logo block matches the top header at 56px (`h-14`)
  for a clean horizontal divider line; expanded logo text bumped to
  `text-base` (16px).

### Architectural decisions

- **Native Rust computation engine** (no WASM, no WebWorkers) invoked via
  Tauri commands. Rayon for data parallelism.
- **Embedded DuckDB** as the only data store; no external database, no
  network round-trip for queries.
- **Tauri v2** as the application shell (no Next.js, no SSR, no API routes).
- **TanStack Router** for type-safe client-side routing with search-param
  validation; **TanStack Query** for all Tauri command data fetching.
- **Trading deferred to post-v1.0.** Paper, live, and bot trading are
  documented as a long-term roadmap rather than silently excluded; the
  data model, command surface, and UI shell stay forward-compatible.

### Known limitations

- No installer or binary distribution. This release is a developer build
  intended for the project author; no GitHub Release will be published for
  `v0.0.1`.
- No real data sources wired up. Tiingo, FRED, and Polygon adapters land in
  Phase 1.
- Bundle size (~498 KB / 154 KB gzipped) exceeds the 250 KB Phase 0 target
  in `route-conventions.md` because no `*.lazy.tsx` route splits exist yet.
  Lazy splitting lands as Recharts-heavy domains start importing in Phases
  3 and beyond.
- Sidebar collapse state lives in React context only; persistence to DuckDB
  via the `app_settings` table is deferred to Phase 1 along with the generic
  `get_setting` / `set_setting` command pair.
- Placeholder app icons generated from a solid Carbon-blue source PNG; real
  branding lands later.

[Unreleased]: https://github.com/chris-c-thomas/chrdfin/compare/v0.0.1...HEAD
[0.0.1]: https://github.com/chris-c-thomas/chrdfin/releases/tag/v0.0.1
