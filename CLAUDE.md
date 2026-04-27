# CLAUDE.md — chrdfin Personal Financial Intelligence Platform

## Project Overview

chrdfin is a comprehensive personal financial intelligence desktop application built with Tauri v2, Vite, React 19, native Rust computation, and DuckDB. It consolidates portfolio backtesting, Monte Carlo simulation, portfolio tracking, optimization, financial calculators, market data, screeners, and news into a single native desktop workstation. The system is designed for a single power user with deep financial and technical expertise.

**Canonical reference:** See `docs/technical-blueprint.md` for the full architectural specification.

**This is NOT just a backtesting tool.** The platform is a full-suite financial workstation. Every architectural decision, package boundary, and naming convention reflects this broader scope.

**This is NOT a web application.** It is a native desktop app built with Tauri v2. There is no Next.js, no server components, no API routes, no Vercel, no Neon PostgreSQL, no WASM, no WebWorkers. The computation engine is native Rust. The database is embedded DuckDB. The frontend is a Vite-powered React SPA rendered in the system webview.

---

## Documentation

The full specification lives in `docs/`. Read `docs/agent-handoff.md` first — it routes you to the right specialized doc based on the task at hand and is the canonical entry point for any non-trivial work.

| Doc | Purpose |
|---|---|
| `docs/agent-handoff.md` | Router for all other docs. Read first. |
| `docs/technical-blueprint.md` | System architecture (canonical) |
| `docs/phase-0-checklist.md` | Phase 0 implementation tasks |
| `docs/database-schema-reference.md` | DuckDB schema |
| `docs/type-definitions-reference.md` | Shared types in `@chrdfin/types` |
| `docs/ui-design-system.md` | Color tokens, typography, density |
| `docs/ui-component-recipes.md` | UI primitives and hooks |
| `docs/chart-recipes.md` | Recharts patterns |
| `docs/route-conventions.md` | TanStack Router structure |
| `docs/data-fetching-patterns.md` | TanStack Query + Tauri events |
| `docs/form-patterns.md` | React Hook Form + Zod |

---

## Repository Structure

```
chrdfin/
├── CLAUDE.md                                # This file — agent instructions
├── docs/
│   ├── technical-blueprint.md               # Full technical specification
│   ├── phase-0-checklist.md                 # Phase 0 implementation guide
│   ├── type-definitions-reference.md        # All TypeScript types
│   └── database-schema-reference.md         # DuckDB schema definitions
│
├── turbo.json
├── pnpm-workspace.yaml
├── package.json
├── .env.example
├── tsconfig.base.json
├── Cargo.toml                               # Workspace-level Cargo config
├── Cargo.lock                               # Tracked — deterministic builds
│
├── apps/
│   └── desktop/                             # Tauri v2 application
│       ├── src/                             # React SPA (Vite-powered)
│       │   ├── main.tsx                     # React entry point
│       │   ├── App.tsx                      # Root component with router
│       │   ├── globals.css                  # Tailwind directives only
│       │   ├── routes/                      # TanStack Router route definitions
│       │   │   ├── __root.tsx               # Root layout (shell wrapper)
│       │   │   ├── index.tsx                # Dashboard home
│       │   │   ├── analysis/                # Backtest, Monte Carlo, Optimizer
│       │   │   ├── tracking/                # Portfolio, Transactions, Watchlist
│       │   │   ├── tools/                   # Calculators, Compare
│       │   │   └── market/                  # Screener, Ticker, Options, News, Calendar
│       │   ├── components/
│       │   │   ├── shell/                   # Platform shell (sidebar, header, command palette)
│       │   │   ├── shared/                  # Cross-domain shared components
│       │   │   └── providers/               # Context providers (theme, Tauri state)
│       │   ├── hooks/                       # useTauriCommand, useProgress, etc.
│       │   └── lib/                         # Utilities
│       │
│       ├── src-tauri/                       # Tauri Rust backend
│       │   ├── Cargo.toml                   # Depends on chrdfin-core
│       │   ├── tauri.conf.json
│       │   ├── capabilities/                # Tauri v2 permission capabilities
│       │   ├── icons/
│       │   └── src/
│       │       ├── main.rs                  # Entry point, plugin registration
│       │       ├── commands/                # Tauri command handlers
│       │       │   ├── compute.rs           # Backtest, MC, optimization
│       │       │   ├── data.rs              # Price queries, search, macro
│       │       │   ├── portfolio.rs         # Portfolio CRUD, holdings, transactions
│       │       │   ├── sync.rs              # Data sync (Tiingo, FRED, RSS)
│       │       │   ├── quotes.rs            # Real-time quote polling
│       │       │   ├── news.rs              # News fetch and query
│       │       │   ├── calculator.rs        # Saved calculator states
│       │       │   └── system.rs            # Settings, DB management, export/import
│       │       ├── db.rs                    # DuckDB connection management
│       │       ├── sync/                    # Provider adapters
│       │       │   ├── tiingo.rs
│       │       │   ├── fred.rs
│       │       │   ├── polygon.rs
│       │       │   └── rss.rs
│       │       ├── state.rs                 # Managed state (DB, config)
│       │       └── error.rs                 # Error types
│       │
│       ├── index.html
│       ├── vite.config.ts
│       └── tailwind.config.ts
│
├── packages/
│   ├── @chrdfin/types                       # Shared TypeScript interfaces & Zod schemas
│   ├── @chrdfin/ui                          # Shared UI component library (shadcn/ui based)
│   ├── @chrdfin/charts                      # Chart wrapper components & configurations
│   ├── @chrdfin/config                      # Shared config, constants, feature flags
│   ├── @chrdfin/tsconfig                    # Shared TypeScript configurations
│   └── @chrdfin/eslint-config               # Shared ESLint configurations
│
├── crates/
│   └── chrdfin-core/                        # Rust computation engine
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs                       # Public API
│           ├── backtest.rs
│           ├── monte_carlo.rs
│           ├── optimizer.rs
│           ├── stats.rs
│           ├── portfolio.rs
│           ├── matrix.rs
│           ├── calculators.rs
│           └── types.rs
│
└── scripts/
    └── seed-data.ts                         # Optional CLI seeding tool
```

---

## Tech Stack & Versions

| Technology | Version | Notes |
|---|---|---|
| Node.js | >= 24.x LTS | Required for Turborepo, Vite |
| pnpm | >= 9.x | Workspace protocol |
| Turborepo | >= 2.4 | Incremental builds, caching |
| Tauri | 2.x | Desktop application framework |
| Vite | >= 6.x | Frontend build tool |
| React | 19.2.x | UI library |
| TypeScript | >= 5.7 | Strict mode everywhere |
| Tailwind CSS | 4.x | CSS-first config |
| TanStack Router | latest | Type-safe client-side routing |
| TanStack Query | latest | Async state management for Tauri commands |
| Rust | >= 1.83 (stable) | Edition 2024 |
| DuckDB | >= 1.x | Via `duckdb-rs` crate |
| Rayon | 1.x | Data parallelism |
| Zod | 3.x | Schema validation, runtime type safety |
| Vitest | 2.x | TypeScript testing |
| ESLint | 9.x | Flat config format |

---

## Package Scope & Naming

All internal packages use the `@chrdfin/*` scope.

```
@chrdfin/types      (leaf — no internal deps)
@chrdfin/config     (leaf — no internal deps)

@chrdfin/charts     -> @chrdfin/types, @chrdfin/ui
@chrdfin/ui         -> @chrdfin/types, @chrdfin/config

apps/desktop        -> all packages
```

**Packages that do NOT exist (removed from web architecture):**

- `@chrdfin/compute` — WASM loader. Replaced by native Rust via Tauri commands.
- `@chrdfin/engine` — TypeScript fallback engine. Unnecessary with native Rust.
- `@chrdfin/data` — Drizzle ORM + Neon driver. Replaced by DuckDB in the Rust backend.

**Enforcement:** ESLint import boundary rules prevent packages from importing against the dependency flow.

---

## Feature Domains & Routes

The platform is organized into feature domains, each isolated in its own route module. Domains are gated by feature flags in `@chrdfin/config`. The Dashboard is the home page and sits outside the section taxonomy — it is rendered above all section groups in the sidebar and is always visible.

Sidebar order (top → bottom): **Dashboard → Tracking → Analysis & Tools → Market → Reference.** Tracking precedes Analysis & Tools intentionally — the day-to-day power-user flow starts at "what do I own and how is it doing?" before reaching for backtesting/MC tooling. Reference sits at the bottom because it is read-only documentation rather than active workflow tooling.

The sidebar uses **plural labels** (`Portfolios`, `Watchlists`, `Screeners`, `Calendars`) for the multi-instance domains. Each plural label routes to a list of saved instances with a "Create" action; when more than one instance exists, the sidebar item gains an inline dropdown chevron so the user can jump straight to a saved instance. See `docs/technical-blueprint.md` § Multi-Instance Domains for the full UX evolution.

| Domain | Sidebar label | Route Path | Feature Flag | Status |
|---|---|---|---|---|
| Dashboard | `Dashboard` | `/` | (always on) | Phase 0 placeholder; widget framework lands in Phase 11 |
| Portfolios | `Portfolios` | `/tracking/portfolio` (+ `$id`) | `tracker` | Phase 5 — multiple per user, classified `tracked`/`backtest`/`model`/`watchlist`/`paper` |
| Transactions | `Transactions` | `/tracking/transactions` | `tracker` | Phase 5 |
| Watchlists | `Watchlists` | `/tracking/watchlist` (+ `$id`) | `tracker` | Phase 5 — multiple per user |
| Backtesting | `Backtesting` | `/analysis/backtest` | `backtest` | Phase 2-3 |
| Monte Carlo | `Monte Carlo` | `/analysis/monte-carlo` | `monteCarlo` | Phase 4 |
| Optimizer | `Optimizer` | `/analysis/optimizer` | `optimizer` | Phase 9 — mean-variance, efficient frontier, risk parity |
| Allocation Optimizer | `Allocation Optimizer` | `/analysis/allocation-optimizer` | `allocationOptimizer` | Phase 9 — rebalancing trades, tax-aware; pairs with Optimizer + Backtest |
| Calculators | `Calculators` | `/tools/calculators/*` | `calculators` | Phase 6 |
| Comparison Tool | `Compare` | `/tools/compare` | `backtest` | Phase 10 |
| Screeners | `Screeners` | `/market/screener` (+ `$id`) | `marketData` | Phase 7 — multiple saved screener configs |
| Ticker Detail | — | `/market/ticker/$symbol` | `marketData` | Phase 7 |
| Options Chain | — | `/market/options/$symbol` | `marketData` | Phase 7 |
| News | `News` | `/market/news` | `news` | Phase 8 — multiple saved feed configurations |
| Calendars | `Calendars` | `/market/calendar` | `news` | Phase 8 — multiple saved calendar configurations |
| Reference Library | `Stocks` / `Options` / `Retirement Accounts` / `Estate Planning` / `Taxes` / `Guides` | `/reference/*` | `reference` | Phase 12 — bundled curated guides |
| Personal Research | — | (TBD) | `research` | Deferred; user-curated saved articles + notes (distinct from Reference Library) |
| Paper Trading *(post-1.0)* | — | (TBD) | `paperTrading` | Post-v1.0 — see Trading Module |
| Live Trading *(post-1.0)* | — | (TBD) | `liveTrading` | Post-v1.0 — broker integrations |
| Bot Trading *(post-1.0)* | — | (TBD) | `botTrading` | Post-v1.0 — algorithmic execution |

**Rule:** Domains never import from each other. Cross-domain data flows through Tauri commands and shared types in `@chrdfin/types`. The Dashboard is a strict consumer — its widgets read from existing domain query surfaces but never import from another domain's `routes/` directory.

**Dashboard vision:** the home page will become a customizable widget grid covering markets overview, portfolio summary, recent backtests, accounts, news, and the earnings/economic calendar. Layout, widget selection, and refresh cadence are user-configurable and persisted in DuckDB (`app_settings.dashboard_layout`). See `docs/technical-blueprint.md` § Dashboard Module for the full spec.

**Trading roadmap (post-1.0):** paper trading, live trading via broker integrations, and bot/algorithmic execution are explicitly **planned** for after the main application is stable. They are not in scope for the current phases but the data model, command surface, and UI shell are designed to accommodate them. See `docs/technical-blueprint.md` § Trading Module for the architecture targets.

---

## Development Commands

```bash
# Development
pnpm tauri dev                  # Start Tauri with Vite HMR + Rust backend
pnpm dev                        # Start only TypeScript packages in dev mode

# Build
pnpm tauri build                # Build desktop app for current platform
pnpm build                      # Build all TypeScript packages

# Quality
pnpm typecheck                  # TypeScript checks across all packages
pnpm lint                       # ESLint across all packages
pnpm lint:fix                   # ESLint with autofix
pnpm format                     # Prettier format
pnpm format:check               # Prettier check

# Generate the TanStack Router tree before standalone typecheck/lint on a fresh checkout:
pnpm --filter desktop exec vite build  # produces apps/desktop/src/routeTree.gen.ts

# Testing (TypeScript)
pnpm test                       # Vitest across all packages
pnpm test:watch                 # Vitest in watch mode

# Testing (Rust)
cargo test                      # All Rust tests
cargo test -p chrdfin-core      # Only core computation tests
cargo nextest run               # Faster parallel test runner (if installed)

# Rust Quality
cargo check                     # Type check
cargo clippy -- -D warnings     # Lints (deny all warnings)
cargo fmt --check               # Format check

# Database (via Tauri commands or CLI)
# DuckDB is managed by the Rust backend — no separate migration tool
```

---

## Code Conventions

### TypeScript

- Strict mode everywhere (`"strict": true` in base tsconfig).
- Prefer `interface` over `type` for object shapes.
- Use `satisfies` operator for type-safe object literals.
- All public functions must have JSDoc comments.
- Prefer named exports over default exports.
- Use `readonly` for immutable properties.
- Exhaustive switch/case with `never` for discriminated unions.

### Naming

- Files: `kebab-case.ts` for modules, `PascalCase.tsx` for React components.
- Variables/functions: `camelCase`.
- Types/interfaces: `PascalCase`.
- Constants: `SCREAMING_SNAKE_CASE` for true constants, `camelCase` for derived values.
- Zod schemas: `PascalCaseSchema` (e.g., `BacktestConfigSchema`, `TransactionInputSchema`).
- Database tables: `snake_case`.
- Rust modules/functions: `snake_case`.
- Rust types/structs: `PascalCase`.
- Package scope: `@chrdfin/*` for all internal packages.

### Imports

- Use workspace protocol for internal packages: `@chrdfin/types`, `@chrdfin/ui`, etc.
- Sort imports: React -> external packages -> internal packages -> relative imports.
- Use path aliases defined in tsconfig (`@/` for app-local imports in `apps/desktop`).

### React

- All components are client components (no server components in Tauri SPA).
- Colocate components with their route when route-specific.
- Shared components go in `@chrdfin/ui` (primitives) or `apps/desktop/src/components` (composites).
- Use React Hook Form + Zod resolver for all forms.
- Use TanStack Router search params for shareable configurations.
- Use TanStack Query for all Tauri command data fetching.
- Use `@tauri-apps/api/event` for real-time event listeners (quotes, progress, sync).
- React 19 has no global `JSX` namespace. For return-type annotations use `import { type JSX } from "react"`.

### Styling

- Always reference `docs/ui-design-system.md` and `docs/ui-component-recipes.md`.
- Tailwind CSS 4 utility classes only. No custom CSS files except for global resets.
- Use CSS variables for design tokens defined in `@chrdfin/config`.
- shadcn/ui components are the base. Extend via composition, not modification.
- Use `cn()` utility (from `@chrdfin/ui/lib/utils`) for conditional class merging.
- Do NOT set `html { font-size }` in `globals.css` — Tailwind utilities are rem-based and inherit the user-agent default (16px). Overriding it silently scales every utility.
- Tailwind v4 has no `text-md` utility (scale jumps from `text-base` 16px to `text-lg` 18px). Use the standard `xs/sm/base/lg/xl/2xl` scale.

### Testing

- Rust tests: inline `#[cfg(test)]` modules. Use `cargo test`.
- TypeScript test files colocated with source: `foo.ts` -> `foo.test.ts`.
- Use `describe` / `it` blocks. Test names should read as sentences.
- Numerical tests must specify tolerance (typically 0.01% for financial calculations).

### Error Handling

- Use Zod `.safeParse()` for input validation on the TypeScript side.
- Tauri commands return `Result<T, String>`. Use `thiserror` in Rust for typed errors.
- Use `Result<T, E>` pattern in Rust computation code (no panics in hot paths).

### Git

- Conventional commits: `feat:`, `fix:`, `chore:`, `docs:`, `test:`, `refactor:`.
- Scope commits to domains: `feat(tracker): add transaction entry form`.
- One logical change per commit.
- All PRs must pass CI (typecheck, lint, test, cargo check, cargo test).
- **Git posture:** the user runs `git commit` / `push` / `tag` / `gh pr create` themselves. Claude prepares (write code, `git add`, draft message), reports, then stops. When the user explicitly asks Claude to commit/push/tag, use `git commit -S` (signed) and `git tag -s` (signed annotated) — never lightweight tags, never `--no-gpg-sign`. No `Co-Authored-By:` trailers.

---

## Environment Variables

Required environment variables (see `.env.example`):

```bash
# Data Provider — Massive (rebranded from Polygon.io on 2025-10-30)
MASSIVE_API_KEY=your_massive_api_key

# Optional rate-limit + history-clamp profile. Defaults to "free" when unset.
# MASSIVE_TIER=free

# App
NODE_ENV=development
```

> **Note:** In the desktop app, API keys are loaded from environment variables during development and from the OS keychain (via Tauri's keyring plugin) or an encrypted config file in production. The `DATABASE_URL` and `DEPLOYMENT_MODE` variables from the web architecture are gone — DuckDB is embedded and requires no connection string.

---

## Database

- DuckDB (embedded, columnar, analytical).
- Managed by the Rust backend in `apps/desktop/src-tauri/src/db.rs`.
- Schema defined as SQL DDL in `docs/database-schema-reference.md`.
- Schema applied programmatically on first app launch (no migration tool needed for single-user).
- Database file location: resolved via Tauri's `app_data_dir()` API.
- Backup: copy the `.duckdb` file, or export to Parquet via the `export_data` command.

### Schema Groups

| Group | Tables | Domain |
|---|---|---|
| Core Market Data | `assets`, `daily_prices`, `dividends`, `macro_series` | Shared |
| Portfolio & Analysis | `portfolios`, `simulation_results` | Backtest, MC, Optimizer |
| Portfolio Tracking | `holdings`, `transactions`, `watchlists` | Tracker |
| News & Research | `news_articles`, `earnings_calendar` | News, Market Data |
| Calculators | `saved_calculator_states` | Calculators |

---

## Current Phase: Phase 0 — Foundation & Tooling

See `docs/phase-0-checklist.md` for the detailed task list with implementation guidance.

**Goal:** A fully configured monorepo with a running Tauri desktop app, DuckDB schema initialized, CI pipeline, and all package stubs. The platform shell (sidebar, header) renders with navigation to all domains.

**Phase 0 is complete when:**

1. `pnpm install` succeeds with no errors.
2. `pnpm typecheck` passes across all packages.
3. `pnpm lint` passes with zero warnings.
4. `pnpm test` runs (even with placeholder tests).
5. `cargo check` passes for all crates.
6. `cargo test` passes for all crates.
7. `pnpm tauri dev` launches the desktop app with the platform shell.
8. All feature domain routes have placeholder pages.
9. Platform shell (sidebar, header) renders with navigation to all domains.
10. Feature flags in `@chrdfin/config` control which domains appear in navigation.
11. DuckDB schema is initialized on app launch with all tables.
12. A placeholder Tauri command (`health_check`) returns a response to the frontend.
13. GitHub Actions CI workflow file exists and passes.

---

## Architecture Principles

1. **Plugin-based domains.** Each feature is an isolated domain with its own route module, feature flag, and package boundary. Domains can be enabled/disabled independently.
2. **Unidirectional dependencies.** Packages never import from packages that depend on them. Domains never import from other domains.
3. **Provider-agnostic data layer.** Data providers implement a common Rust `DataProvider` trait. Swapping providers requires only a new adapter.
4. **Native computation.** All heavy computation runs as native Rust with Rayon parallelism. No WASM, no TypeScript fallback, no WebWorkers.
5. **Schema-first validation.** All inputs have Zod schemas (TypeScript) and serde validation (Rust). Defense in depth.
6. **Local-first data.** All data is stored in a local DuckDB file. No cloud database dependency.
7. **Shared data contracts.** Cross-domain data flow uses `PortfolioContext` and other shared types from `@chrdfin/types`.

---

## Data Provider Strategy

| Provider | Required | Cost | Used For |
|---|---|---|---|
| **Massive** (Polygon.io rebrand) | Yes | Free tier (5 RPM, ~2y history) → Stocks Starter $29/mo and up | EOD prices, ticker reference, dividends, splits, treasury yields, inflation, market status |
| **RSS Feeds** | Optional | Free | Supplementary news headlines (Phase 8) |

Start on the free tier during development. Tier is pinned via `MASSIVE_TIER` env var (defaults to `free`); flip to `paid` once on Stocks Starter or higher to unlock the higher rate limit and longer history. Real-time quotes (Phase 5) and options chains (Phase 7) require a paid tier.

---

## What NOT to Do

- Do NOT install a global state management library (Redux, Jotai, Zustand, etc.). Use React Context + `useReducer` scoped to domain routes.
- Do NOT use `any` type. Use `unknown` and narrow.
- Do NOT import between feature domains. Cross-domain data flows through Tauri commands and `@chrdfin/types`.
- Do NOT commit `.env` files. Only `.env.example` is tracked.
- Do NOT use `f32` in Rust computation code. Use `f64` throughout for numerical precision.
- Do NOT add features not in the current phase's deliverable list without explicit approval.
- Do NOT use default exports. Use named exports everywhere.
- Do NOT add CSS files. Use Tailwind utility classes only.
- Do NOT use `@backtest/*` as a package scope. The correct scope is `@chrdfin/*`.
- Do NOT implement broker integrations, live trading, paper trading, or bot/algorithmic execution **during the current development phases** (0 through 12). These capabilities are explicitly part of the long-term roadmap (post-v1.0) and the data model, command surface, and UI shell are designed to accommodate them — but writing any actual trading code, broker adapter, or order-routing logic before v1.0 is shipped and stable is out of scope. See `docs/technical-blueprint.md` § Trading Module for the architecture targets.
- Do NOT store API keys in the database or in plaintext files. Use environment variables (dev) or OS keychain (prod).
- Do NOT use Next.js, server components, API routes, or any server-side rendering patterns. This is a Tauri SPA.
- Do NOT use Drizzle ORM, PostgreSQL, Neon, or any external database. DuckDB is the database.
- Do NOT use WASM, wasm-pack, wasm-bindgen, or WebWorkers. Computation is native Rust.
- Do NOT use SWR. Use TanStack Query for Tauri command data fetching.
- Do NOT use `nuqs`. Use TanStack Router search params.
- Do NOT use Comlink. There are no WebWorkers to communicate with.
- Do NOT use `localStorage` or `sessionStorage`. Use DuckDB (via Tauri commands) for persistent state.
- Do NOT panic in Rust computation code. Use `Result<T, E>` everywhere.

---

## Common Gotchas

- **Tauri `generate_handler!` paths:** Use full module paths (`commands::system::foo`), not re-exported names — the macro looks for the synthetic `__cmd__<name>` helper next to the function definition.
- **TanStack Router `redirect()` + Zod `.default()`:** Routes whose `validateSearch` has default-bearing fields require `search: Schema.parse({})` on programmatic redirects.
- **Shared library tsconfigs must not declare `rootDir`** — it resolves relative to the file declaring it, breaking every consuming package. Per-package tsconfigs override if needed.
- **Tauri icons:** `pnpm exec tauri icon <source.png>` (run from `apps/desktop`) generates the full icon set from a single 1024×1024 source PNG.
- **Cargo workspace `resolver = "3"`** is required for edition 2024 (already locked in `Cargo.toml`).
- **Tauri `[lib].crate-type` is `["cdylib", "rlib"]` only** — never re-add `staticlib`. It produces a multi-hundred-MB archive that exhausts CI runner disk and isn't consumed anywhere.
- **`Icon?` rule in `.gitignore` matches `icons/` case-insensitively on APFS** — the macOS legacy desktop-icon pattern hides the Tauri icon set on default-config Macs. The `!apps/desktop/src-tauri/icons/**` force-include at the bottom of `.gitignore` is load-bearing; don't remove it.
- **`pnpm/action-setup@v4` + `packageManager` field:** never pass `with: { version: ... }` — the action reads the exact version from `package.json` `packageManager` and errors with `ERR_PNPM_BAD_PM_VERSION` when both are set.
- **CI debugging:** use `gh run list --limit 5` and `gh run view <id> --log-failed` to pull failed-step logs straight to the terminal instead of clicking through the GitHub web UI.
- **Massive (Polygon rebrand):** free tier is **5 RPM** with ~2 years of history on aggregates. Default `MASSIVE_TIER` is `Free`; flip to `Paid` (or `starter` / `developer` / `advanced` / `business`) to unlock 100 req/sec and history back to 1990-01-01.
- **Massive macro endpoints return bundles:** `/fed/v1/treasury-yields`, `/fed/v1/inflation`, and `/fed/v1/labor-market` each return one row carrying every tenor / flavor at once. The adapter in `sync/massive/mappers.rs` explodes them into per-`MacroSeriesId` observations so `macro_series` stays a clean `(series_id, date) -> value` shape.
- **Splits adjustment:** Massive returns `split_from` / `split_to` ratios. Backtest math should multiply historical share counts by `split_to / split_from` (a 4-for-1 split is `split_from=1, split_to=4`).
- **HTTP egress goes through `AppHttpClient`:** never `fetch()` from the webview. The Rust client owns the bearer header, gzip, and the timeout policy, and keeps API keys off the JS side.
- **Tauri command bodies factor into `*_inner` functions:** the `#[tauri::command]` wrapper grabs `state.db` / `state.sync` and delegates. Integration tests target the inner functions directly so they don't need a Tauri runtime — see `tests/commands_data.rs`.
- **Provider DTOs use `serde(rename_all = "camelCase")`:** wire format is camelCase so structs round-trip cleanly to TypeScript over Tauri IPC. Storage helpers map columns to Rust fields manually, so the rename only affects JSON serialization.
- **`vitest.config.ts` must duplicate `vite.config.ts`'s `@/` alias:** vitest doesn't inherit Vite's `resolve.alias`. Test imports of `@/lib/...` fail with "Failed to resolve import" until the alias is also declared in the vitest config.
- **`Sync` struct shadows `std::marker::Sync`:** when adding a trait bound that needs the auto trait, write `+ std::marker::Sync` in full. The `ProgressFn` type alias in `sync/orchestrator.rs` is the canonical example.

---

## Releases & Versioning

Pre-1.0, the minor version tracks the development phase: `v0.N.0` = Phase N completion, `v0.N.M` = patch within Phase N. `v0.0.0` is invalid SemVer; Phase 0 ships as `v0.0.1`. Releases are tag-driven (no auto-release on push to main). Full policy in `.claude/instructions/changelog-and-releases.md`.
