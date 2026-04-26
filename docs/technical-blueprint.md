# Technical Blueprint

## chrdfin — Personal Financial Intelligence Platform

---

**Version 3.0** | April 2026 | *Status: Pre-1.0 development*

| | |
|---|---|
| **Architecture** | Tauri v2 + Vite + React SPA + Turborepo + TypeScript |
| **Computation** | Native Rust (multi-threaded via Rayon) |
| **Data Storage** | DuckDB (embedded, columnar) |
| **Deployment** | Desktop application (macOS, Linux, Windows) |
| **Platform Scope** | Dashboard, Backtesting, Monte Carlo, Portfolio Tracking, Optimization, Allocation Optimization, Calculators, Market Data, News, Reference Library, Research, future Trading roadmap (paper / live / bot) |

---

## Table of Contents

- [Executive Summary](#executive-summary)
- [Platform Vision & Feature Domains](#platform-vision--feature-domains)
- [Key Architectural Decisions](#key-architectural-decisions)
- [Decision Register](#decision-register)
- [Plugin Architecture](#plugin-architecture)
- [Monorepo Architecture](#monorepo-architecture)
- [Data Acquisition & Storage](#data-acquisition--storage)
- [Computation Engine](#computation-engine)
- [Backtesting Engine](#backtesting-engine)
- [Monte Carlo Simulation Engine](#monte-carlo-simulation-engine)
- [Dashboard Module](#dashboard-module)
- [Multi-Instance Domains](#multi-instance-domains)
- [Portfolio Tracking Module](#portfolio-tracking-module)
- [Portfolio Optimization Module](#portfolio-optimization-module)
- [Allocation Optimizer Module](#allocation-optimizer-module)
- [Financial Calculators Module](#financial-calculators-module)
- [Market Data & Screeners Module](#market-data--screeners-module)
- [News & Calendars Module](#news--calendars-module)
- [Reference Library Module](#reference-library-module)
- [Trading Module](#trading-module)
- [Frontend Architecture](#frontend-architecture)
- [Tauri Command Layer](#tauri-command-layer)
- [Development Phases](#development-phases)
- [Testing Strategy](#testing-strategy)
- [Build & Distribution](#build--distribution)
- [Security Considerations](#security-considerations)
- [Future Web Platform Path](#future-web-platform-path)
- [Risk Register](#risk-register)
- [Dependency Inventory](#dependency-inventory)
- [Appendix: Glossary](#appendix-glossary)

---

## Executive Summary

chrdfin is a comprehensive personal financial intelligence desktop application that consolidates portfolio analysis, backtesting, simulation, optimization, tracking, market data, and research into a single native workstation. The platform draws inspiration from Bloomberg Terminal, FactSet, Morningstar Direct, and Koyfin — distilled into a personal tool built for a single power user with deep financial and technical expertise.

The application is built with Tauri v2 (Rust backend + system webview frontend), giving it direct access to system resources — all CPU cores, all available memory, and the local filesystem — without the constraints of a browser sandbox. The computation engine runs as native Rust with Rayon-based data parallelism, and data is stored locally in DuckDB, a columnar analytical database purpose-built for the exact query patterns financial time series analysis demands.

The system is architected around a **plugin-based feature domain model** where each major capability (backtesting, portfolio tracking, calculators, screeners, news) is an independent module that plugs into shared infrastructure (data layer, computation engine, UI shell). New features can be added without modifying existing modules.

Core capabilities, delivered incrementally:

1. **Portfolio Backtesting** — Deterministic historical backtesting with configurable rebalancing strategies across equities, ETFs, mutual funds, and cash equivalents with 30+ years of data.
2. **Monte Carlo Simulation** — Stochastic forward-looking probabilistic analysis using parametric (GBM), historical bootstrap, and block bootstrap methods. Native Rust with Rayon enables 1M+ iterations.
3. **Portfolio Tracking** — Manual position entry with holdings, transactions, cost basis, and real-time P&L. **Multiple portfolios per user**, classified as `tracked` (real holdings), `backtest` (test portfolios), `model` (target allocations), `paper` (paper-trading), or named `watchlist` lists. Broker integrations (read-only) added later.
4. **Portfolio Optimization** — Mean-variance optimization, efficient frontier visualization, risk parity, Black-Litterman, and factor-based allocation tools.
5. **Allocation Optimization** — A focused tool for optimizing target allocations and rebalancing trades against backtest history, tax constraints, and a chosen rebalancing strategy. Pairs with the Optimizer and the Backtest engine.
6. **Financial Calculators** — Compound growth, retirement planning, withdrawal strategy comparison, tax-loss harvesting simulation, options payoff diagrams, risk/reward calculators.
7. **Market Data & Screeners** — Real-time (during market hours) and historical price data for equities, ETFs, mutual funds, and options chains. **Multiple saved screener configurations** per user.
8. **News & Calendars** — Aggregated financial news feeds, earnings calendars, and economic event calendars. **Multiple saved feed and calendar configurations** per user.
9. **Reference Library** — Bundled guides on Stocks, Options, Retirement Accounts, Estate Planning, Taxes, and other financial topics. Read-only knowledge base; expands as the platform matures.
10. **Personal Research** — A user-curated research library (saved articles, notes, bookmarks). Distinct from the curated Reference Library.
11. **Customizable Dashboard** — A widget-based home screen pulling from every other domain. Layout, widget selection, and refresh cadence are user-configurable.
12. **Trading (post-1.0 roadmap)** — Paper trading, live trading via broker integrations, and bot/algorithmic trading are explicitly **planned** for a phase after the main application is built and stable. They are not in scope for the current phases but the data model, command surface, and UI shell are designed to accommodate them. See [Trading Module](#trading-module).

The architecture prioritizes computational power, local data sovereignty, and the keyboard-driven, information-dense experience that professional financial workstations deliver. Live, paper, and bot trading are deferred to a post-1.0 phase — they are explicitly part of the long-term plan, not silently excluded.

---

## Platform Vision & Feature Domains

### Domain Map

Each feature domain is a self-contained vertical that depends on shared horizontal infrastructure. Domains never import from each other directly — they communicate through the shared data layer and shared types.

```
┌─────────────────────────────────────────────────────────────────────┐
│                       chrdfin Platform Shell                        │
│  (App Layout, Navigation, Command Palette, Global Search, Themes)   │
├─────────────────────────────────────────────────────────────────────┤
│           Customizable Dashboard (widgets, home screen)             │
├──────────┬──────────┬──────────┬──────────┬──────────┬──────────────┤
│ Backtest │ Monte    │ Portfolio│ Optimizer │Financial │ Market Data  │
│ Engine   │ Carlo    │ Tracker  │ + Alloc   │Calculators│ & Screeners │
│          │ Sim      │          │ Optimizer │          │              │
├──────────┴──────────┴──────────┴──────────┴──────────┴──────────────┤
│           News & Calendars  │  Reference Library  │  Research        │
├─────────────────────────────────────────────────────────────────────┤
│   Trading Module (post-1.0): Paper · Live (broker) · Bot · Alerts    │
├─────────────────────────────────────────────────────────────────────┤
│                  Shared Infrastructure Layer                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐              │
│  │ @chrdfin/ │ │ @chrdfin/ │ │ @chrdfin/ │ │ @chrdfin/ │              │
│  │ types    │ │ ui       │ │ charts   │ │ config   │              │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘              │
├─────────────────────────────────────────────────────────────────────┤
│               Tauri v2 Rust Backend                                  │
│  ┌────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────────┐ │
│  │chrdfin-core│ │ DuckDB   │ │ Data     │ │ Background Sync      │ │
│  │(compute)   │ │ (storage)│ │ Adapters │ │ (Tiingo/FRED/RSS)    │ │
│  └────────────┘ └──────────┘ └──────────┘ └──────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘
```

### Feature Domain Definitions

| Domain | Description | Data Dependencies | Compute Requirements |
|---|---|---|---|
| **Dashboard** | Widget-based home screen aggregating signals from every other domain | All domain queries via the shared layer | Light (composition + cached query results) |
| **Backtesting** | Historical portfolio simulation with rebalancing strategies | Daily prices, dividends, macro series | Heavy (native Rust, multi-threaded) |
| **Monte Carlo** | Forward-looking probabilistic simulation | Historical returns, macro series | Heavy (native Rust, Rayon parallelism) |
| **Portfolio Tracker** | Multiple portfolios (tracked, backtest, model, paper, watchlist), holdings, transactions, cost basis, P&L | Daily prices, user portfolio data | Light (Rust query + TS rendering) |
| **Optimizer** | Mean-variance, efficient frontier, risk parity, Black-Litterman | Daily prices, covariance matrices | Medium-Heavy (native Rust) |
| **Allocation Optimizer** | Optimize target allocations and rebalancing trades for backtests, real portfolios, and tax constraints | Daily prices, holdings, target weights, tax lot data | Medium (native Rust; reuses optimizer + backtest internals) |
| **Calculators** | Compound growth, retirement, withdrawals, options payoff, tax | User inputs, macro series | Light (Rust) |
| **Market Data & Screeners** | Price quotes, charts, options chains, multi-instance screener configurations | Real-time + historical prices, fundamentals | Light (Rust query layer) |
| **News & Calendars** | Multi-instance news feeds, earnings calendars, economic event calendars | External news APIs, earnings data | Light (Rust HTTP + TS rendering) |
| **Reference Library** | Bundled curated guides on Stocks, Options, Retirement Accounts, Estate Planning, Taxes, etc. | Static content shipped with the app (Markdown / MDX) | Light (no compute) |
| **Personal Research** | User-curated saved articles, notes, bookmarks (distinct from Reference Library) | User content + linked tickers | Light (TS query + persistence) |
| **Trading** *(post-1.0)* | Paper trading, live trading via broker integrations, bot/algorithmic execution | Real-time quotes, order book, broker APIs | Variable (per strategy); see Trading Module |

### Cross-Domain Interactions

Domains share data through the Rust backend's query layer but never call each other's internal functions. Examples of cross-domain data flow:

- Portfolio Tracker holdings feed into Backtest as a starting allocation.
- Backtest results feed into Monte Carlo as a historical return series.
- Optimizer produces target allocations that can be compared to Tracker's current allocation.
- **Allocation Optimizer reads target weights from any portfolio (`tracked`, `backtest`, `model`, `paper`) and produces a set of optimized rebalancing trades, optionally accounting for tax lots and turnover constraints.** Outputs can feed back into Backtest for what-if validation.
- Market Data provides the real-time prices that Tracker uses for current P&L.
- News feed items can be linked to tickers in any of the Tracker's watchlists.
- Dashboard widgets read from every domain's query surface but contribute nothing back — it is strictly a consumer, never a producer. Each widget is a thin React component bound to a TanStack Query result; widget code never imports from another domain's feature code.
- Reference Library is a read-only consumer of bundled content; it never reads from or writes to other domains.
- Trading Module (post-1.0) plugs into Tracker for portfolio context, Market Data for execution-time quotes, and the Optimizer / Allocation Optimizer for strategy inputs. It writes back trade fills as Transactions.

These flows are mediated by shared types and Tauri commands, not by direct inter-module coupling. A `PortfolioContext` type in `@chrdfin/types` serves as the common data contract.

---

## Key Architectural Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Application Shell | Tauri v2 | Rust backend with system webview. Native performance, small binary, direct hardware access. |
| Frontend Framework | Vite + React 19 SPA | Fast HMR, no server component complexity. Shares packages with potential future web app. |
| Monorepo Tooling | Turborepo + pnpm | Fast incremental builds, workspace isolation, native pnpm support. |
| Client-Side Router | TanStack Router | Type-safe routing, search param validation, file-based route generation. |
| Computation Engine | Native Rust (Rayon) | Direct access to all CPU cores and memory. No WASM serialization overhead. |
| Primary Data Provider | Tiingo API | Best cost/coverage ratio. 30+ years EOD data. Stocks, ETFs, mutual funds. |
| Options Data Provider | CBOE DataShop / Polygon.io | Options chains, IV surface, Greeks. Polygon for real-time during market hours. |
| News Data Provider | Tiingo News API + RSS aggregation | Tiingo News included with API key. Supplemented with curated RSS feeds. |
| Database | DuckDB (embedded) | Columnar analytical database. Ideal for time-series queries. Zero network latency. No server process. |
| Charting | Lightweight Charts + Recharts | TradingView engine for time series. Recharts for statistical visualizations. |
| Package Scope | `@chrdfin/*` | Consistent namespace tied to existing CHRD brand identity. Publishable to npm. |
| Plugin Model | Feature Flag + Route Modules | Each domain is a route module gated by a feature flag in `@chrdfin/config`. Disabled domains produce zero client-side bundle. |

---

## Decision Register

### DR-001: Computation Strategy

**Decision:** Native Rust computation engine with Rayon-based data parallelism, invoked directly from Tauri commands. No WASM, no WebWorkers, no TypeScript fallback.

**Context:** Portfolio backtests over 30 years of daily data for 10+ assets produce approximately 75,000+ data points per simulation. Monte Carlo simulations multiply this by 1,000 to 1,000,000+ iterations. Portfolio optimization requires covariance matrix computation and quadratic programming. The target machines (AMD 9950X3D with 128GB RAM, Mac Studio M4 Max with 64GB unified memory) have significant compute resources that a browser sandbox cannot fully exploit.

#### Alternatives Considered

| Approach | Pros | Cons | Verdict |
|---|---|---|---|
| Rust/WASM in WebWorkers | Browser-portable, same code web+desktop | Memory caps, serialization overhead, limited parallelism | Rejected — unnecessary for desktop-only |
| Pure TypeScript | Simpler toolchain, no Rust dependency | JS performance ceiling, single-threaded | Rejected — insufficient for heavy computation |
| **Native Rust (Rayon)** | Full hardware access, true data parallelism, no serialization, direct memory allocation | Requires Rust toolchain | **Selected** |
| GPU compute (CUDA/Metal) | Massive parallelism for Monte Carlo | Platform-specific, complex programming model | Future consideration |

**Implementation:** The `chrdfin-core` crate contains all computation logic. Tauri commands in `apps/desktop/src-tauri/` expose these as async commands to the frontend. Long-running operations report progress via Tauri events. The same Rust crate could later compile to WASM if a web frontend is added.

### DR-002: Data Provider Selection

**Decision:** Tiered data provider strategy. Tiingo as the primary historical/EOD provider. Polygon.io as an optional upgrade for real-time intraday and options data. FRED for macroeconomic data. Tiingo News API + curated RSS for news feeds.

| Provider | Coverage | Cost | Quality | Role |
|---|---|---|---|---|
| **Tiingo** | 30+ years EOD, stocks/ETFs/mutual funds, adj. prices, dividends, splits, news | Free: 500 req/hr, 50 symbols/day. Power: $10/mo unlimited. | High. Corporate actions properly adjusted. | **Primary — EOD, metadata, news** |
| **Polygon.io** | Full market, real-time capable, options data, reference data | Free: 5 calls/min. Starter: $29/mo. Full: $199/mo. | Excellent. Institutional grade. | **Optional — real-time, options chains** |
| **CBOE DataShop** | Options chains, IV surface, historical options | Varies. Some free delayed data. | Excellent for options. | **Options (if Polygon not used)** |
| **FRED** | Treasury rates, CPI, Fed Funds, economic series | Free. No meaningful rate limits. | Authoritative. | **Macro data** |
| **Tiingo News** | Financial news, blog posts, press releases | Included with Tiingo API key | Good. Covers major outlets. | **Primary news feed** |
| **RSS Feeds** | Bloomberg, Reuters, CNBC, WSJ, FT, Seeking Alpha | Free (public RSS endpoints) | Varies. Headlines + summaries. | **Supplementary news** |

**Strategy:** Start with Tiingo free tier during development. Upgrade to Tiingo Power ($10/mo) for production. Add Polygon.io Starter ($29/mo) when real-time intraday prices and options chains are needed. Data is cached locally in DuckDB after initial fetch, so API calls are only needed for syncs, real-time quotes, and new ticker additions.

The data ingestion layer is provider-agnostic via a Rust `DataProvider` trait. Adding a new provider requires only a new adapter implementation with no changes to consuming code.

### DR-003: Database Selection

**Decision:** DuckDB (embedded columnar database).

**Rationale:** DuckDB is an embedded analytical database optimized for exactly the query patterns chrdfin requires: range scans over date-indexed time series, aggregations across tickers, and analytical window functions. Key advantages over PostgreSQL for a desktop application:

- **Zero network latency.** No TCP round-trips. Database is an in-process library call.
- **No server process.** No PostgreSQL daemon to manage, no connection pooling, no port conflicts.
- **Columnar storage.** Time-series queries that scan `adj_close` across 30 years of data read only that column, not the entire row. Dramatically faster for analytical workloads.
- **Native Parquet support.** Import/export data as Parquet files for backup, sharing, and bulk loading.
- **Analytical SQL.** First-class window functions, CTEs, and aggregate operations.
- **Single-file database.** One file on disk. Easy to backup, move, or reset.
- **Rust bindings.** `duckdb-rs` provides native integration with the Tauri backend.

**Storage location:** `~/.chrdfin/data/chrdfin.duckdb` (XDG-compliant on Linux, `~/Library/Application Support/chrdfin/` on macOS, `%APPDATA%\chrdfin\` on Windows). Resolved via Tauri's `app_data_dir()` API.

### DR-004: Charting Strategy

**Decision:** Dual-library approach using Lightweight Charts (TradingView) for financial time series and Recharts for statistical and dashboard visualizations.

**Rationale:** Lightweight Charts is purpose-built for financial data — handles millions of data points efficiently, provides native candlestick/line/area chart types, supports crosshair synchronization, and has a small bundle footprint (~40KB gzipped). Recharts covers statistical visualizations: histograms, bar charts, pie/donut charts, scatter plots, and treemaps.

### DR-005: Frontend Routing

**Decision:** TanStack Router for client-side routing.

**Rationale:** In a Tauri SPA, there is no server to handle routing — all routing is client-side. TanStack Router provides type-safe route definitions with validated search parameters (replacing `nuqs` which is Next.js-specific), file-based route generation, and first-class TypeScript support. Route-level code splitting via lazy routes ensures disabled feature domains produce zero bundle impact.

### DR-006: Real-Time Data Architecture

**Decision:** Polling-based real-time updates during market hours via Tauri background commands.

**Rationale:** A 15-second polling interval to Polygon.io (or Tiingo IEX) during market hours (9:30 AM - 4:00 PM ET, weekdays) provides sufficient quote freshness for portfolio tracking and market monitoring. Polling runs as a background Rust task in the Tauri backend, emitting results to the frontend via Tauri events. This avoids browser CORS restrictions entirely.

---

## Plugin Architecture

### Domain Registration

Each feature domain registers itself with the platform shell through a standard `DomainManifest` interface:

```typescript
// packages/@chrdfin/types/src/platform.ts

interface DomainManifest {
  readonly id: string;
  readonly name: string;
  readonly description: string;
  readonly icon: string;
  readonly basePath: string;
  readonly navigationItems: NavigationItem[];
  readonly featureFlag: string;
  readonly dependencies: string[];
}
```

### Feature Flags

```typescript
// packages/@chrdfin/config/src/features.ts

export const FEATURES = {
  backtest: true,
  monteCarlo: true,
  tracker: true,
  optimizer: false,
  calculators: true,
  marketData: true,
  news: true,
  research: false,
} as const satisfies Record<string, boolean>;

export type FeatureId = keyof typeof FEATURES;

export function isFeatureEnabled(id: FeatureId): boolean {
  return FEATURES[id] ?? false;
}
```

### Cross-Domain Communication

Domains communicate through shared data contracts, never through direct imports. Data flow between domains is always mediated by Tauri commands that query DuckDB. The Optimizer reads the current portfolio from the same `portfolios` table the Tracker writes to — it never calls Tracker code directly.

---

## Monorepo Architecture

### Directory Layout

```
chrdfin/
├── turbo.json
├── pnpm-workspace.yaml
├── package.json
├── .env.example
├── tsconfig.base.json
├── Cargo.toml                               # Workspace-level Cargo config
├── Cargo.lock                               # Tracked for deterministic builds
├── CLAUDE.md
│
├── docs/
│   ├── technical-blueprint.md
│   ├── phase-0-checklist.md
│   ├── type-definitions-reference.md
│   └── database-schema-reference.md
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
│       │   │   ├── analysis/
│       │   │   │   ├── backtest.tsx
│       │   │   │   ├── backtest.$id.tsx
│       │   │   │   ├── monte-carlo.tsx
│       │   │   │   ├── monte-carlo.$id.tsx
│       │   │   │   └── optimizer.tsx
│       │   │   ├── tracking/
│       │   │   │   ├── portfolio.tsx
│       │   │   │   ├── portfolio.$id.tsx
│       │   │   │   ├── transactions.tsx
│       │   │   │   └── watchlist.tsx
│       │   │   ├── tools/
│       │   │   │   ├── calculators.tsx
│       │   │   │   ├── calculators.compound-growth.tsx
│       │   │   │   ├── calculators.retirement.tsx
│       │   │   │   ├── calculators.withdrawal.tsx
│       │   │   │   ├── calculators.options-payoff.tsx
│       │   │   │   ├── calculators.tax-loss.tsx
│       │   │   │   └── compare.tsx
│       │   │   └── market/
│       │   │       ├── screener.tsx
│       │   │       ├── ticker.$symbol.tsx
│       │   │       ├── options.$symbol.tsx
│       │   │       ├── news.tsx
│       │   │       └── calendar.tsx
│       │   ├── components/
│       │   │   ├── shell/
│       │   │   ├── shared/
│       │   │   └── providers/
│       │   ├── hooks/
│       │   └── lib/
│       │
│       ├── src-tauri/                       # Tauri Rust backend
│       │   ├── Cargo.toml
│       │   ├── tauri.conf.json
│       │   ├── capabilities/
│       │   ├── icons/
│       │   └── src/
│       │       ├── main.rs
│       │       ├── commands/
│       │       │   ├── mod.rs
│       │       │   ├── compute.rs
│       │       │   ├── data.rs
│       │       │   ├── portfolio.rs
│       │       │   ├── sync.rs
│       │       │   ├── quotes.rs
│       │       │   ├── news.rs
│       │       │   ├── calculator.rs
│       │       │   └── system.rs
│       │       ├── db.rs
│       │       ├── sync/
│       │       │   ├── mod.rs
│       │       │   ├── tiingo.rs
│       │       │   ├── fred.rs
│       │       │   ├── polygon.rs
│       │       │   └── rss.rs
│       │       ├── state.rs
│       │       └── error.rs
│       │
│       ├── index.html
│       ├── vite.config.ts
│       ├── tailwind.config.ts
│       └── package.json
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
│   └── chrdfin-core/                        # Rust computation + data engine
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
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
    └── seed-data.ts                         # Optional: initial data seeding via CLI
```

**Packages removed from web architecture:** `@chrdfin/compute` (WASM loader), `@chrdfin/engine` (TypeScript fallback), `@chrdfin/data` (Drizzle ORM + Neon). The data layer moves entirely into Rust.

### Package Dependency Graph

```
@chrdfin/types      (leaf — no internal deps)
@chrdfin/config     (leaf — no internal deps)

@chrdfin/charts     -> @chrdfin/types, @chrdfin/ui
@chrdfin/ui         -> @chrdfin/types, @chrdfin/config

apps/desktop        -> @chrdfin/types, @chrdfin/ui, @chrdfin/charts, @chrdfin/config
```

**Enforcement:** ESLint import boundary rules prevent packages from importing against the dependency flow.

### Minimum Package Versions

| Package | Version | Notes |
|---|---|---|
| Node.js | >= 22.x LTS | Required for native fetch, Turborepo |
| pnpm | >= 9.x | Workspace protocol, catalogs support |
| Turborepo | >= 2.4 | Latest stable |
| React | 19.2.x | View Transitions, useEffectEvent, Activity, React Compiler (stable) |
| TypeScript | >= 5.7 | Satisfies operator, strict mode |
| Tailwind CSS | 4.x | CSS-first config, Lightning CSS engine |
| Vite | >= 6.x | Fast HMR, native ESM |
| Rust | >= 1.83 (stable) | Edition 2024 |
| Tauri CLI | >= 2.x | Tauri v2 |
| DuckDB | >= 1.x | Via `duckdb-rs` crate |

---

## Data Acquisition & Storage

### Data Requirements

| Data Category | Source | Granularity | History | Update Frequency | Used By |
|---|---|---|---|---|---|
| Equity prices (adj. close, OHLCV) | Tiingo | Daily EOD | 30+ years | Daily (market close) | Backtest, Tracker, Market Data |
| ETF prices (adj. close, OHLCV) | Tiingo | Daily EOD | Since inception | Daily | Backtest, Tracker, Market Data |
| Mutual fund NAVs | Tiingo | Daily EOD | 30+ years | Daily | Backtest, Tracker |
| Real-time quotes | Polygon.io / Tiingo IEX | 15s poll | Current day | Real-time during market hours | Tracker, Market Data |
| Options chains | Polygon.io | Snapshot | Current + 1 year historical | Intraday | Options viewer, Calculators |
| Dividends & distributions | Tiingo | Event-based | 30+ years | As declared | Backtest, Tracker |
| Risk-free rate (T-Bill / Fed Funds) | FRED | Daily | 60+ years | Daily | Backtest, MC, Optimizer, Calculators |
| Inflation (CPI-U) | FRED | Monthly | 70+ years | Monthly | Calculators, MC |
| Earnings dates & estimates | Polygon.io / Tiingo | Event-based | Current + 2 quarters | Daily | Calendar, Market Data |
| Financial news | Tiingo News API | Article-based | 30 days rolling | Every 15 minutes | News feed |
| RSS news headlines | Bloomberg, Reuters, etc. | Article-based | Current day | Every 5 minutes | News feed |
| Ticker metadata | Tiingo | Static | Current | Weekly refresh | All domains |

### Data Ingestion Pipeline

The ingestion system is implemented entirely in Rust within the Tauri backend. Each provider implements a common `DataProvider` trait.

#### Ingestion Flow

1. **Seed Phase (one-time):** Run initial data sync via a Tauri command. Bulk-fetches historical data for ~500 tickers. Rate-limited with exponential backoff. Progress reported via Tauri events.

2. **Daily Sync (background task):** A Rust background thread runs on app launch and on schedule. Fetches latest prices after market close (6:00 PM ET). Upserts into DuckDB.

3. **Real-Time Polling (background task):** During market hours, a Rust async task polls quotes every 15 seconds for tickers in the active view. Results emitted via Tauri events.

4. **On-Demand Fetch:** When a user adds a new ticker, the system fetches its full history on demand and stores it.

5. **News Sync (background task):** Runs every 15 minutes during market hours. RSS feeds fetched directly by the Rust backend (no CORS issues).

6. **FRED Sync:** Monthly background task updates macroeconomic series.

### Database Schema

DuckDB schema designed for the full platform scope. See `docs/database-schema-reference.md` for complete schema definitions.

#### Estimated Storage

| Table | Rows (500 tickers) | Est. Size (500) | Est. Size (5,000 tickers) |
|---|---|---|---|
| daily_prices | ~3.75M | ~150 MB | ~1.5 GB |
| dividends | ~100K | ~4 MB | ~40 MB |
| macro_series | ~50K | ~2 MB | ~2 MB |
| assets | ~500 | < 1 MB | ~1 MB |
| Other tables | Varies | < 20 MB | < 80 MB |
| **Total** | | **~170 MB** | **~1.6 GB** |

DuckDB's columnar compression typically achieves 3-5x over row-oriented databases for time-series data. A 500-ticker universe fits trivially. Even 5,000 tickers stays under 2GB.

---

## Computation Engine

The computation engine is the mathematical core of the platform. It is implemented as a native Rust library with Rayon-based data parallelism.

### Architecture Overview

```
React UI --> Tauri invoke() --> Rust Command Handler --> chrdfin-core
                                      |                       |
                                      |-- Progress events --> UI updates
                                      |-- DuckDB queries  --> Data loading
```

No serialization boundary between the command handler and computation core — they share the same process, memory space, and Rust types. Data flows from DuckDB into `chrdfin-core` as native Rust types, not JSON strings.

### Tauri Command Interface

```rust
// apps/desktop/src-tauri/src/commands/compute.rs

#[tauri::command]
async fn run_backtest(
    state: tauri::State<'_, AppState>,
    window: tauri::Window,
    config: BacktestConfig,
) -> Result<BacktestResult, String> {
    let db = state.db.lock().await;
    let price_data = db.load_prices(&config.tickers, &config.start, &config.end)?;

    let result = tokio::task::spawn_blocking(move || {
        chrdfin_core::run_backtest(config, price_data, |progress| {
            let _ = window.emit("backtest:progress", &progress);
        })
    }).await.map_err(|e| e.to_string())?;

    result.map_err(|e| e.to_string())
}
```

### Rust Crate Modules

```
crates/chrdfin-core/src/
├── lib.rs              # Public API surface
├── backtest.rs         # Daily-stepping backtest engine
├── monte_carlo.rs      # GBM, historical bootstrap, block bootstrap (Rayon)
├── optimizer.rs        # Mean-variance, risk parity, efficient frontier
├── stats.rs            # Sharpe, Sortino, CAGR, max drawdown, VaR, CVaR
├── portfolio.rs        # Return calculation, rebalancing, dividend reinvestment
├── matrix.rs           # Covariance matrix, Cholesky decomposition
├── calculators.rs      # Financial calculators (pure functions)
└── types.rs            # Shared Rust types with serde serialization
```

### Performance Targets

With native Rust + Rayon on target hardware:

- **9950X3D (32 threads):** 1,000,000 MC iterations, 10-asset, 30-year: < 5 seconds.
- **M4 Max (14 cores):** Same workload: < 8 seconds.
- **Backtest (10 assets, 30 years):** < 100ms.

---

## Backtesting Engine

### Output Metrics

| Metric | Formula | Description |
|---|---|---|
| CAGR | `(end/start)^(1/years) - 1` | Compound Annual Growth Rate |
| Total Return | `(end - start) / start` | Total percentage return |
| Max Drawdown | `max(peak - trough) / peak` | Worst peak-to-trough decline |
| Sharpe Ratio | `(Rp - Rf) / σp` | Risk-adjusted return (annualized) |
| Sortino Ratio | `(Rp - Rf) / σd` | Downside risk-adjusted return |
| Standard Deviation | `σ(daily returns) * √252` | Annualized volatility |
| Best/Worst Year | Annual return extremes | Calendar year returns |
| VaR (95%) | 5th percentile of daily returns | Daily Value at Risk |
| CVaR (95%) | Mean of returns below VaR | Expected Shortfall |
| Calmar Ratio | `CAGR / |Max Drawdown|` | Return per unit of drawdown |
| Ulcer Index | `√(mean(D²))` where D = drawdown | Sustained drawdown severity |

---

## Monte Carlo Simulation Engine

### Simulation Methods

1. **Parametric (GBM):** Log-normal return generation. `dS = μSdt + σSdW`. Fast, assumes normal distribution.
2. **Historical Bootstrap:** Random sampling with replacement from actual daily returns. Preserves fat tails and skewness.
3. **Block Bootstrap:** Random sampling of contiguous blocks (default: 63 trading days). Preserves autocorrelation.

---

## Dashboard Module

The dashboard is the application's home screen and the route a user lands on after launch. It is rendered at `/` and is the only nav item in the sidebar that lives outside the four domain section groups — it is conceptually the entry point, not a domain.

### Vision

A customizable grid of widgets giving the user an at-a-glance overview of every other domain. Layout (which widgets, where, what size), refresh cadence, and per-widget configuration (e.g. which tickers a "Market Overview" widget tracks) are all user-configurable and persisted in DuckDB so the dashboard restores between sessions.

### Initial widget set

These are the targets for the first iteration of the widget framework. Every widget is a thin React component bound to a TanStack Query result reading from existing Tauri commands; no domain feature code is duplicated.

| Widget | Reads from | Notes |
|---|---|---|
| **Portfolio Summary** | `list_portfolios`, `list_holdings`, `get_quotes` | Total value, day change, allocation breakdown, top movers across selected portfolios. |
| **Market Overview** | `get_quotes` (watchlist), `get_macro_series` | Major indices, sector performance, configurable watchlist quotes during market hours. |
| **Recent Backtests** | `simulation_results` query | Last-run portfolio simulations with CAGR, max drawdown, and Sharpe summary. |
| **Accounts** | `list_portfolios`, `list_holdings` | Aggregated balances across linked accounts and tracked portfolios. |
| **News** | `get_news` | Top headlines and ticker-tagged stories from Tiingo News and curated RSS feeds. |
| **Earnings & Calendar** | `earnings_calendar` query, `get_macro_series` | Upcoming earnings releases and economic events for tracked tickers. |
| **Quick Actions** | (none) | Buttons to launch a new backtest, create a portfolio, search a ticker, etc. |

### Architecture constraints

1. **Strictly a consumer.** Dashboard widgets only read from existing domain query surfaces. They never own data, never import from another domain's `routes/` directory, and never bypass the shared Tauri command layer.
2. **Widget framework precedes widgets.** A small `<DashboardGrid>` + `<Widget>` system in `apps/desktop/src/dashboard/` (or a future `@chrdfin/dashboard` package) handles drag/drop, resize, and config persistence before any widget ships.
3. **Layout persisted to DuckDB.** The `app_settings` table stores layout state under a `dashboard_layout` key. No localStorage. Cross-machine sync via the existing Parquet export/import path.
4. **Phase 0 placeholder.** Until the framework lands, the route renders an intent placeholder listing the planned widgets and the Phase 0 IPC health check.
5. **Always visible in the sidebar.** Unlike domain nav items, the Dashboard nav entry has no feature flag — it is the home page and is rendered above all section groups in `apps/desktop/src/components/shell/sidebar.tsx`.

### Implementation phase

Slotted as **Phase 11** below — after the core domains have shipped enough commands and data shapes for the widgets to bind to. The widget framework can be moved earlier if the placeholder home view starts blocking real workflows; the trade-off is that early widgets would have less to display.

---

## Multi-Instance Domains

Several domains in chrdfin host *multiple named instances per user*, not single-page experiences. The sidebar surfaces these with **plural** labels (`Portfolios`, `Watchlists`, `Screeners`, `Calendars`) to make the multi-instance nature visible from the entry point.

### Affected domains

| Domain | Sidebar label | Instance examples |
|---|---|---|
| Portfolios | `Portfolios` | A real `tracked` book; multiple `backtest` test portfolios; several `model` target allocations; one or more `paper` portfolios (post-1.0) |
| Watchlists | `Watchlists` | "Tech megacaps", "Dividend aristocrats", "AI/ML names", "Earnings this week" — each a named ticker list |
| Screeners | `Screeners` | "Small-cap value with FCF growth", "High-yield ETFs", "Reasonable P/E tech" — saved filter sets |
| Calendars | `Calendars` | "My earnings calendar" (filtered to held tickers), "Macro calendar" (FOMC, CPI, NFP), "IPO watch", "Splits & dividends" |
| News | `News` | Multiple curated feed configurations (e.g. "Macro only", "Held tickers only", "Sector deep dives") |

### UX evolution

- **Phase ≤ N (initial ship):** Each plural sidebar item routes to a *list page* of the user's saved instances, with a primary "Create" action and links to detail views.
- **Phase ≤ N + 1 (refinement):** When a domain has more than one saved instance, the sidebar item gains a **dropdown chevron** that expands inline to show the saved instances. Clicking the parent label still navigates to the list page; clicking an expanded child navigates straight to that instance.
- **Phase ≤ N + 2 (power-user features):** Drag-to-reorder instances, pin favorites to the dashboard, group by tag.

### Data model

Instance multiplicity is already in the schema — every multi-instance domain has its own table (`portfolios`, `watchlists`, `news_articles` queryable by saved filter, etc.) keyed by `id` UUID. The `portfolio_type` column distinguishes portfolio classes (`tracked`, `backtest`, `model`, `watchlist`, `paper`).

### Cross-instance interactions

- The Backtest engine accepts any `portfolio_id` as a starting allocation (any classification).
- The Allocation Optimizer can target any `portfolio_id` for rebalancing trades.
- The Dashboard's Portfolio Summary widget supports per-instance selection.
- Watchlists feed the News Feed and Calendar instances as ticker filters.

---

## Portfolio Tracking Module

### Core Features

1. **Multiple Portfolios:** Users create as many portfolios as they need, classified by `portfolio_type` — `tracked` (real holdings), `backtest` (test portfolios), `model` (target allocations only, no holdings), `watchlist` (named ticker lists, no holdings), `paper` (paper-trading; post-1.0). The schema column is `VARCHAR` so additional types can be appended without a migration.
2. **Holdings Management:** Add/edit/remove positions with cost basis. Multiple lots per ticker. Scoped per portfolio.
3. **Transaction History:** Full audit trail of buys, sells, dividends, splits, and transfers per portfolio.
4. **Real-Time P&L:** Updated via Tauri event-driven quote polling during market hours, per portfolio.
5. **Allocation View:** Breakdown by asset class, sector, and holding with drift analysis.
6. **Performance History:** Computed from holdings + daily_prices via DuckDB.
7. **Multiple Watchlists:** Real-time quote table with configurable columns. Users save many watchlists with arbitrary tickers; each is its own row in `watchlists`.

---

## Portfolio Optimization Module

| Method | Description | Phase |
|---|---|---|
| Mean-Variance (Markowitz) | Maximize return for given risk | Phase 9 |
| Efficient Frontier | Full frontier of optimal portfolios | Phase 9 |
| Risk Parity | Equalize risk contribution from each asset | Phase 9 |
| Black-Litterman | Market equilibrium + investor views | Phase 9 |
| Maximum Sharpe | Tangency portfolio | Phase 9 |
| Minimum Volatility | Global minimum variance portfolio | Phase 9 |

The Optimizer answers *"given a universe of assets and a risk model, what's the optimal weight vector?"*. For *"given a current portfolio and a target weight vector, what trades should I make?"* see the [Allocation Optimizer Module](#allocation-optimizer-module).

---

## Allocation Optimizer Module

A focused tool that pairs with both the Optimizer (which produces target weights) and the Backtester (which validates the resulting series). The sidebar presents it as a sibling of the Optimizer in the **Analysis & Tools** section.

### Purpose

Given a current portfolio (any `portfolio_type`) and a target allocation, produce an optimized set of rebalancing trades that minimizes a configurable cost function:

- Tracking error vs. target weights
- Realized capital gains tax (per `lot` selection method)
- Transaction costs and slippage (configurable)
- Turnover (percent of portfolio traded)

### Inputs

| Input | Source |
|---|---|
| Current holdings | `holdings` table for the selected `portfolio_id` |
| Target weights | Output of the Optimizer, a `model` portfolio, or manual entry |
| Tax-lot data | `transactions` table with FIFO/LIFO/specific-ID lot selection |
| Constraints | Min/max trade size, no-short, no-buy-X-after-sell wash-sale window, etc. |

### Outputs

- Recommended trade list (ticker, side, shares, expected proceeds/cost)
- Estimated realized gain/loss after the trades
- Resulting drift from target weights
- Optional: feed the post-rebalance portfolio back into the Backtest engine for what-if validation

### Phase

Lands alongside the Optimizer in **Phase 9**. The two share matrix utilities (`chrdfin-core::matrix`) and the optimization solver.

---

## Financial Calculators Module

| Calculator | Description |
|---|---|
| **Compound Growth** | Project future value with contributions |
| **Retirement Planner** | Multi-phase retirement projection |
| **Withdrawal Strategy** | Compare withdrawal approaches |
| **Options Payoff** | Visualize P&L for options strategies |
| **Tax-Loss Harvesting** | Estimate after-tax benefit |
| **Risk/Reward** | Quick entry/exit analysis |
| **Position Size** | Kelly criterion and fixed-risk sizing |
| **Margin Calculator** | Reg-T and portfolio margin estimates |
| **DCA vs. Lump Sum** | Compare investment timing strategies |

All calculators implemented as pure Rust functions in `chrdfin-core::calculators`.

---

## Market Data & Screeners Module

The screener engine is powered by DuckDB's analytical query engine — filtering and sorting across thousands of rows with multiple predicates is a columnar database's core strength.

The sidebar exposes this domain as **Screeners** (plural). Users save multiple named screener configurations — each a row in a `saved_screeners` table (added when the domain ships). The sidebar item routes to a list of saved screeners with a "New screener" action; the dropdown UX (per the Multi-Instance Domains section) opens individual screeners directly.

---

## News & Calendars Module

The sidebar exposes two plural items — **News** and **Calendars** — backed by per-instance saved configurations.

### News Feed Architecture

1. **Tiingo News API:** Fetched by Rust backend, stored in DuckDB.
2. **RSS Feeds:** Fetched directly by Rust backend (no CORS issues).
3. **Filtering:** By ticker, source, date range, bookmark status.
4. **Saved feed configurations:** Each combination of filters is a named `news_feed` instance. Examples: "Macro only", "Held tickers only", "Sector deep dives".

### Calendars

| Calendar type | Source |
|---|---|
| Earnings | `earnings_calendar` table, filtered by ticker scope |
| Economic events | FRED + curated event list (FOMC, CPI, NFP, jobs, GDP) |
| IPO watch | Tiingo / Polygon IPO calendar |
| Dividends & splits | Tiingo corporate actions |

Each saved calendar is a named filter configuration over the underlying tables.

### Curated RSS Feed Sources

| Source | Content Type |
|---|---|
| Bloomberg Markets | Markets, macro |
| Reuters Business | Business, earnings |
| CNBC | Market news, analysis |
| Financial Times | Global markets, opinion |
| Seeking Alpha | Analysis, stock picks |
| Federal Reserve | Fed statements, minutes |
| Calculated Risk | Housing, macro |

---

## Reference Library Module

A bundled, curated knowledge base distinct from the user-curated Personal Research domain. The Reference Library ships read-only guides written and maintained by the project; users consume but do not edit them.

### Initial topic set

| Topic | Coverage |
|---|---|
| Stocks | Equity instruments, order types, corporate actions, dividends, splits, ADRs, ETFs, mutual funds, how chrdfin models each |
| Options | Calls, puts, multi-leg strategies, Greeks (delta/gamma/theta/vega/rho), implied volatility, assignment, expiration, tax treatment |
| Retirement Accounts | Traditional IRA, Roth IRA, 401(k), 403(b), 457, HSA, SEP, SIMPLE, Solo 401(k); contribution limits, RMDs, conversions, withdrawals |
| Estate Planning | Wills, trusts, beneficiary designations, gift and estate tax exemptions, step-up in basis, multi-generational portfolio modelling |
| Taxes | Capital gains (short vs long), wash sales, qualified vs ordinary dividends, tax-loss harvesting, lot selection (FIFO, LIFO, specific ID), AMT, state-specific notes |

The list is open-ended; new topics can be added without changing the navigation surface beyond appending an item to the Reference section.

### Architecture

- Content is shipped as **bundled Markdown / MDX files** under `apps/desktop/src/reference/` (or a future `@chrdfin/reference` package). No DuckDB rows for content.
- Cross-links to live data are allowed: a guide on options can embed a live IV chart for a user-selected ticker via the same chart components used in the Market Data domain.
- Search across all reference content lives in the global command palette under a `Reference` group.
- Versioning travels with the app — content updates ship in app releases, not at runtime.

### Phase

Implementation lands in **Phase 12** alongside or just after the dashboard widget framework. Initial content can be stubbed and filled in over time; the Phase 12 deliverable is the shell + search + first iteration of each topic.

---

## Trading Module *(post-1.0)*

The trading domain is **planned but explicitly deferred** until the main application is built and stable. It is documented here so the data model, command surface, and UI shell stay forward-compatible — not because it is in the immediate scope.

### Three trading modes

1. **Paper trading.** Simulated order execution against real-time market quotes. No real money, no broker. Uses the existing `paper` portfolio classification. Surfaces realistic fills, partial fills, slippage, and rejection scenarios.
2. **Live trading.** Real order routing through one or more broker integrations. Read-only broker connectivity ships first (account balances, positions, transactions) and write capabilities (order placement, cancellation, modification) are added behind explicit per-account opt-in and additional security gates.
3. **Bot / algorithmic trading.** User-defined or library-provided strategies running as background tasks in the Rust backend. Strategies can fire against either paper or live accounts; safety limits, kill switches, and audit logs are mandatory.

### Architecture targets

- **Strategy harness in `chrdfin-core::trading`** with deterministic backtest mode (reuses the Backtest engine's daily stepping).
- **Order book schema** in DuckDB: `orders`, `order_fills`, `strategy_runs`, with foreign keys to `portfolios` and `transactions`.
- **Broker adapters** behind a Rust trait, mirroring the data-provider adapter pattern (one trait, many implementations).
- **UI integration:** orders, fills, and strategy runs surface in the Tracker section; bots get a dedicated "Strategies" sub-section under Tracking.
- **Security:** broker credentials in OS keychain. No credentials in DuckDB. Explicit confirmation dialogs for live order placement above a threshold.

### Phase

Implementation begins after the **stable v1.0 release**. The exact phase numbering is intentionally left open — paper trading may land first as Phase 13, live trading as Phase 14, bot trading as Phase 15, but the order can shift based on user demand.

---

## Frontend Architecture

### State Management

- **No global state library.** React Context + `useReducer` scoped to domain routes.
- **Server state:** TanStack Query for Tauri command data fetching.
- **Route state:** TanStack Router search params for shareable configurations.
- **Form state:** React Hook Form + Zod resolver.
- **Tauri events:** `@tauri-apps/api/event` for real-time updates.

### Frontend-Backend Communication

```typescript
// apps/desktop/src/hooks/use-tauri-command.ts
import { invoke } from '@tauri-apps/api/core';
import { useQuery, useMutation } from '@tanstack/react-query';

export function usePrices(tickers: string[], start: string, end: string) {
  return useQuery({
    queryKey: ['prices', tickers, start, end],
    queryFn: () => invoke<PriceData>('get_prices', { tickers, start, end }),
  });
}

export function useRunBacktest() {
  return useMutation({
    mutationFn: (config: BacktestConfig) =>
      invoke<BacktestResult>('run_backtest', { config }),
  });
}
```

---

## Tauri Command Layer

The Tauri command layer replaces REST API routes. Each command is a `#[tauri::command]` function invoked via `invoke()`.

### Command Organization

| Command | Category | Description |
|---|---|---|
| `get_prices` | Data | Historical price data |
| `get_quotes` | Data | Real-time quotes |
| `search_tickers` | Data | Ticker search/autocomplete |
| `get_macro_series` | Data | Macroeconomic time series |
| `get_asset_metadata` | Data | Asset metadata and fundamentals |
| `run_backtest` | Compute | Execute backtest (emits progress events) |
| `run_monte_carlo` | Compute | Execute MC simulation (emits progress events) |
| `optimize_portfolio` | Compute | Run portfolio optimization |
| `compute_efficient_frontier` | Compute | Generate efficient frontier |
| `calculate` | Compute | Run any financial calculator |
| `list_portfolios` | Portfolio | List saved portfolios |
| `create_portfolio` | Portfolio | Create portfolio |
| `update_portfolio` | Portfolio | Update portfolio |
| `delete_portfolio` | Portfolio | Delete portfolio |
| `list_holdings` | Tracker | List holdings |
| `add_holding` | Tracker | Add a position |
| `update_holding` | Tracker | Update a position |
| `remove_holding` | Tracker | Remove a position |
| `list_transactions` | Tracker | Transaction history |
| `add_transaction` | Tracker | Record a transaction |
| `list_watchlists` | Tracker | List watchlists |
| `manage_watchlist` | Tracker | Create/update/delete watchlist |
| `get_news` | News | Query news articles |
| `sync_data` | Sync | Trigger manual data sync |
| `get_sync_status` | Sync | Current sync status |
| `save_calculator_state` | Calculator | Persist calculator inputs |
| `load_calculator_states` | Calculator | List saved states |
| `export_data` | System | Export database as Parquet |
| `import_data` | System | Import data from Parquet |
| `get_db_stats` | System | Database size and table counts |

---

## Development Phases

### Phase Overview

| Phase | Name | Duration | Key Deliverable |
|---|---|---|---|
| 0 | Foundation & Tooling | 1-2 weeks | Monorepo scaffold, Tauri app shell, DuckDB schema, CI |
| 1 | Data Layer | 2-3 weeks | DuckDB schema, Rust Tiingo/FRED adapters, seed, background sync |
| 2 | Computation Core | 2-3 weeks | Native Rust backtest engine, Rayon parallelism |
| 3 | Backtest UI | 2-3 weeks | Portfolio builder, backtest form, results dashboard |
| 4 | Monte Carlo Engine + UI | 2-3 weeks | MC simulation (3 methods), fan chart, distributions |
| 5 | Portfolio Tracker | 2-3 weeks | Multiple portfolios (tracked/backtest/model/watchlist), holdings, transactions, real-time P&L, multiple watchlists |
| 6 | Financial Calculators | 2-3 weeks | All calculators with save/load |
| 7 | Market Data & Screeners | 2-3 weeks | Ticker detail, multiple saved screeners, options chain |
| 8 | News & Calendars | 1-2 weeks | Multiple news feed configurations, multiple calendar configurations, earnings, economic |
| 9 | Portfolio Optimization | 2-3 weeks | Optimizer (mean-variance, efficient frontier, risk parity) and Allocation Optimizer (rebalancing trades, tax-aware) |
| 10 | Polish & Distribution | 2-3 weeks | Command palette, performance, packaging, auto-update |
| 11 | Customizable Dashboard | 2-3 weeks | `<DashboardGrid>` widget framework with drag/drop/resize, layout persisted to DuckDB, initial widget set. Replaces the Phase 0 placeholder home view. |
| 12 | Reference Library | 2-3 weeks | Bundled MDX content for Stocks, Options, Retirement Accounts, Estate Planning, Taxes; cross-links to live data; command palette search |
| **Stable v1.0 release** | — | — | All of the above shipping reliably; no live trading until 1.0 is stable |
| 13+ | Trading (post-1.0) | TBD | Paper trading → live broker integration → bot/algorithmic execution. Each is its own phase; order can shift based on demand. |

**Total estimated timeline: 24-34 weeks to v1.0** (6-8.5 months at 15-20 hrs/week). Trading phases (13+) are intentionally unscheduled.

> Phases 0-4 are the core delivery. Phases 5-12 can be reordered or parallelized; Phase 11 (Dashboard) depends on read-side commands shipped by Phases 1, 5, 7, and 8. Phase 13+ (Trading) does not begin until v1.0 is shipped and stable.

### Phase 0: Foundation & Tooling

**Goal:** A running Tauri desktop app with platform shell, DuckDB schema, CI, and all package stubs.

#### Deliverables

1. Initialize monorepo with pnpm workspaces and Turborepo.
2. Configure shared TypeScript, ESLint, Prettier.
3. Configure Tailwind CSS 4 with shared design tokens.
4. Initialize shadcn/ui in `@chrdfin/ui`.
5. Scaffold Tauri v2 app with Vite + React.
6. Configure TanStack Router with placeholder routes for ALL domains.
7. Build platform shell: sidebar, header, market status.
8. Configure feature flags in `@chrdfin/config`.
9. Set up Rust workspace: root `Cargo.toml`, `chrdfin-core` crate, Tauri app crate.
10. Initialize DuckDB with full schema.
11. Create placeholder Tauri commands.
12. Configure Vitest for TS packages.
13. Set up GitHub Actions CI.
14. Verify `pnpm tauri dev` launches the app.

### Phase 1: Data Layer

#### Deliverables

1. Implement Tiingo adapter in Rust.
2. Implement FRED adapter in Rust.
3. Implement `sync_data` Tauri command with progress events.
4. Implement background sync after market close.
5. Implement data query commands: `get_prices`, `search_tickers`, `get_macro_series`, `get_asset_metadata`.
6. Implement on-demand fetch for unknown tickers.
7. Write tests for providers and DuckDB queries.
8. Run initial seed. Validate integrity.

### Phase 2: Computation Core

#### Deliverables

1. Implement Rust backtest engine (daily stepping, rebalancing, dividends).
2. Implement statistics module (all metrics).
3. Wire `run_backtest` Tauri command with DuckDB loading and progress events.
4. Implement Rayon parallelism.
5. Numerical accuracy tests.
6. Benchmark: target <100ms for 10-asset, 30-year backtest.

### Phase 3: Backtest UI

#### Deliverables

1. `PortfolioBuilder`: ticker search via Tauri command, allocation weights.
2. `BacktestConfigForm`: date range, rebalancing, benchmark, dividends.
3. Results dashboard: metrics grid, equity curve, drawdown, annual returns, allocation.
4. Full flow: form -> invoke -> progress -> results.
5. Route search params.
6. Chart crosshair sync.
7. Benchmark overlay.

### Phase 4: Monte Carlo Engine + UI

#### Deliverables

1. Implement GBM, historical bootstrap, block bootstrap with Rayon.
2. Percentile extraction, terminal distribution.
3. MC config form (up to 1M iterations).
4. Fan chart, histogram, probability of success.
5. Route search params.
6. Numerical validation.

### Phases 5-10

*(Deliverable lists unchanged from v2.0 blueprint, with "API routes" replaced by "Tauri commands" and "SWR" replaced by "TanStack Query" throughout.)*

---

## Testing Strategy

| Layer | Tool | Scope | Location |
|---|---|---|---|
| Unit (Rust) | `cargo test` | Computation, statistics, adapters | Inline `#[cfg(test)]` |
| Unit (TS) | Vitest | Schemas, utilities, config | Colocated `*.test.ts` |
| Component | Vitest + Testing Library | React components, hooks | Colocated `*.test.tsx` |
| Integration (Rust) | `cargo test` | DuckDB queries, Tauri commands | `tests/` in crates |
| E2E | Tauri WebDriver / Playwright | Full user flows | `apps/desktop/e2e/` |
| Numerical | `cargo test` + Vitest | Financial calculations | Dedicated test modules |

---

## Build & Distribution

### Development

```bash
pnpm tauri dev        # Vite HMR + Rust backend
pnpm typecheck        # TypeScript checks
pnpm lint             # ESLint
pnpm test             # Vitest
cargo check           # Rust check
cargo test            # Rust tests
cargo clippy          # Rust lints
```

### Production Build

```bash
pnpm tauri build      # Build for current platform
```

Produces: macOS `.dmg` + `.app`, Windows `.msi` + `.exe`, Linux `.AppImage` + `.deb` + `.rpm`.

### CI Pipeline (GitHub Actions)

1. **TS Quality:** `pnpm install --frozen-lockfile` -> typecheck -> lint -> format:check -> test
2. **Rust Quality:** `cargo check` -> `cargo clippy -- -D warnings` -> `cargo test`
3. **Build:** `pnpm tauri build` (matrix strategy for all platforms)
4. **Release:** Tauri GitHub Action creates draft releases with installers

---

## Security Considerations

- **API Key Storage:** OS keychain via Tauri's `keyring` plugin, or encrypted config file.
- **Database Access:** Local DuckDB file. No network exposure. User-only file permissions.
- **Input Validation:** `serde` on Rust side, Zod on TypeScript side. Defense in depth.
- **IPC Security:** Tauri v2 capability system restricts exposed commands.
- **Network Access:** Rust backend makes all external HTTP requests. Webview has no direct external access.
- **Code Signing:** Production builds should be code-signed (Apple Developer ID, Authenticode).

---

## Future Web Platform Path

The architecture preserves a clean path to a web frontend:

- **Shared packages** (`@chrdfin/types`, `@chrdfin/ui`, `@chrdfin/charts`, `@chrdfin/config`) work in any React environment.
- **`chrdfin-core`** can compile to WASM via `wasm-pack`.
- **A web app** would be `apps/web/` consuming the same packages with PostgreSQL instead of DuckDB.

This is explicitly deferred.

---

## Risk Register

| Risk | Impact | Likelihood | Mitigation |
|---|---|---|---|
| Tiingo API deprecation or pricing change | High | Low | Provider adapter pattern. Local DuckDB preserves all data. |
| Polygon.io cost | Medium | Medium | Polygon is optional. Tiingo IEX for free delayed quotes. |
| Tauri v2 breaking changes | Medium | Low | Pin version. Strong semver commitment. |
| DuckDB breaking changes | Medium | Low | Pin `duckdb-rs` version. Stable file format. |
| Numerical precision errors | High | Low | Reference test suite. `f64` throughout. |
| Data quality issues | High | Medium | Validation on ingestion. Flag anomalies. |
| Scope creep | Medium | High | Strict phase gates. Phases 0-4 first. |
| Cross-machine sync | Low | Medium | Parquet export/import. Syncthing for single-writer. |
| macOS notarization | Low | High | Budget time for Apple Developer ID setup. |

---

## Dependency Inventory

### Core

| Package / Crate | Version | License | Purpose |
|---|---|---|---|
| tauri | 2.x | Apache-2.0/MIT | Desktop framework |
| vite | 6.x | MIT | Frontend build |
| react / react-dom | 19.2.x | MIT | UI library |
| typescript | 5.7.x | Apache-2.0 | Type system |
| tailwindcss | 4.x | MIT | Utility CSS |
| turborepo | 2.4.x | MPL-2.0 | Monorepo builds |

### UI & Charting

| Package | Version | License | Purpose |
|---|---|---|---|
| @radix-ui/* | latest | MIT | Accessible UI primitives |
| shadcn/ui | latest | MIT | Component library |
| lightweight-charts | 4.x | Apache-2.0 | Financial charts |
| recharts | 2.x | MIT | Statistical charts |
| @tanstack/react-table | latest | MIT | Headless tables |
| @tanstack/react-router | latest | MIT | Type-safe routing |
| @tanstack/react-query | latest | MIT | Async state management |
| react-hook-form | 7.x | MIT | Form state |
| zod | 3.x | MIT | Schema validation |
| cmdk | latest | MIT | Command palette |
| lucide-react | latest | ISC | Icons |

### Rust Crates

| Crate | Version | License | Purpose |
|---|---|---|---|
| duckdb | 1.x | MIT | Embedded analytical DB |
| reqwest | 0.12.x | Apache-2.0/MIT | HTTP client |
| tokio | 1.x | MIT | Async runtime |
| rayon | 1.x | Apache-2.0/MIT | Data parallelism |
| serde / serde_json | 1.x | Apache-2.0/MIT | Serialization |
| chrono | 0.4.x | Apache-2.0/MIT | Date/time |
| rand | 0.8.x | Apache-2.0/MIT | RNG |
| statrs | 0.17.x | MIT | Statistics |
| nalgebra | 0.33.x | Apache-2.0 | Linear algebra |
| thiserror | 2.x | Apache-2.0/MIT | Error types |
| tracing | 0.1.x | MIT | Structured logging |

### Development & Testing

| Package | Version | License | Purpose |
|---|---|---|---|
| vitest | 2.x | MIT | TS testing |
| @testing-library/react | latest | MIT | Component testing |
| eslint | 9.x | MIT | Linting |
| prettier | 3.x | MIT | Formatting |

---

## Appendix: Glossary

| Term | Definition |
|---|---|
| **Adjusted Close** | Stock price adjusted for splits, dividends, and distributions. |
| **Black-Litterman** | Portfolio allocation model combining market equilibrium with investor views. |
| **CAGR** | Compound Annual Growth Rate. |
| **CVaR** | Conditional Value at Risk (Expected Shortfall). |
| **DuckDB** | Embedded columnar analytical database for OLAP workloads. |
| **GBM** | Geometric Brownian Motion. Stochastic process for modeling stock prices. |
| **Greeks** | Option price sensitivities: Delta, Gamma, Theta, Vega, Rho. |
| **IV** | Implied Volatility. |
| **Rayon** | Rust data parallelism library. |
| **Sharpe Ratio** | `(Rp - Rf) / σp`. Risk-adjusted return. |
| **Sortino Ratio** | Like Sharpe but uses only downside deviation. |
| **Tauri** | Framework for desktop apps with web frontends and Rust backends. |
| **VaR** | Value at Risk. Maximum expected loss at a given confidence level. |
