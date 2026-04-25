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
| Node.js | >= 22.x LTS | Required for Turborepo, Vite |
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

The platform is organized into feature domains, each isolated in its own route module. Domains are gated by feature flags in `@chrdfin/config`.

| Domain | Route Path | Feature Flag | Status |
|---|---|---|---|
| Backtesting | `/analysis/backtest` | `backtest` | Phase 2-3 |
| Monte Carlo | `/analysis/monte-carlo` | `monteCarlo` | Phase 4 |
| Optimizer | `/analysis/optimizer` | `optimizer` | Phase 9 (deferred) |
| Portfolio Tracker | `/tracking/portfolio` | `tracker` | Phase 5 |
| Transactions | `/tracking/transactions` | `tracker` | Phase 5 |
| Watchlists | `/tracking/watchlist` | `tracker` | Phase 5 |
| Calculators | `/tools/calculators/*` | `calculators` | Phase 6 |
| Comparison Tool | `/tools/compare` | `backtest` | Phase 10 |
| Stock Screener | `/market/screener` | `marketData` | Phase 7 |
| Ticker Detail | `/market/ticker/$symbol` | `marketData` | Phase 7 |
| Options Chain | `/market/options/$symbol` | `marketData` | Phase 7 |
| News Feed | `/market/news` | `news` | Phase 8 |
| Calendar | `/market/calendar` | `news` | Phase 8 |

**Rule:** Domains never import from each other. Cross-domain data flows through Tauri commands and shared types in `@chrdfin/types`.

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

### Styling

- Always reference `docs/ui-design-system.md` and `docs/ui-component-recipes.md`.
- Tailwind CSS 4 utility classes only. No custom CSS files except for global resets.
- Use CSS variables for design tokens defined in `@chrdfin/config`.
- shadcn/ui components are the base. Extend via composition, not modification.
- Use `cn()` utility (from `@chrdfin/ui/lib/utils`) for conditional class merging.

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

---

## Environment Variables

Required environment variables (see `.env.example`):

```bash
# Data Providers
TIINGO_API_KEY=your_tiingo_api_key
FRED_API_KEY=your_fred_api_key

# Optional: Real-time quotes and options data
POLYGON_API_KEY=your_polygon_api_key

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
| **Tiingo** | Yes | $10/mo (Power) | EOD prices, metadata, dividends, news, search |
| **FRED** | Yes | Free | Treasury rates, CPI, macro series |
| **Polygon.io** | Optional | $29/mo (Starter) | Real-time quotes, options chains, fundamentals |
| **RSS Feeds** | Optional | Free | Supplementary news headlines |

Start with Tiingo free tier during development. Tiingo Power ($10/mo) for production.

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
- Do NOT implement broker integrations, live trading, or order execution.
- Do NOT store API keys in the database or in plaintext files. Use environment variables (dev) or OS keychain (prod).
- Do NOT use Next.js, server components, API routes, or any server-side rendering patterns. This is a Tauri SPA.
- Do NOT use Drizzle ORM, PostgreSQL, Neon, or any external database. DuckDB is the database.
- Do NOT use WASM, wasm-pack, wasm-bindgen, or WebWorkers. Computation is native Rust.
- Do NOT use SWR. Use TanStack Query for Tauri command data fetching.
- Do NOT use `nuqs`. Use TanStack Router search params.
- Do NOT use Comlink. There are no WebWorkers to communicate with.
- Do NOT use `localStorage` or `sessionStorage`. Use DuckDB (via Tauri commands) for persistent state.
- Do NOT panic in Rust computation code. Use `Result<T, E>` everywhere.
