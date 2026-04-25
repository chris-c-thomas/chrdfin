# Type Definitions Reference

All TypeScript interfaces and Zod schemas for `@chrdfin/types`. These are defined in Phase 0 as the shared type foundation for all packages.

## Changes from v2.0 (Web Architecture)

- **Removed:** `api.ts` — No REST API routes. Tauri commands handle all data flow.
- **Removed:** `providers.ts` — `DataProvider` is now a Rust trait, not a TypeScript interface.
- **Removed:** `Observable` from `common.ts` — Tauri events replace Comlink progress streams.
- **Updated:** `compute.ts` — Removed `ComputeService` interface (replaced by direct Tauri `invoke()` calls). Kept `PortfolioMetrics`, `RebalancingStrategy`, `StrategyContext` types as these are used for display and form validation.
- **Updated:** `calculators.ts` — `maxMCIterations` increased to 1,000,000 (native Rust handles this).
- **All other files:** Unchanged from v2.0.

## File Organization

```
packages/types/src/
├── index.ts              # Barrel re-export of everything
├── common.ts             # Result type, utility types, ProgressEvent
├── market-data.ts        # Price data, assets, dividends, quotes, options
├── portfolio.ts          # Portfolio allocation, config, context
├── backtest.ts           # Backtest config and results
├── monte-carlo.ts        # Monte Carlo config and results
├── tracker.ts            # Holdings, transactions, watchlists
├── optimizer.ts          # Optimization config and results
├── calculators.ts        # Calculator config/result types for all calculators
├── news.ts               # News articles, earnings calendar, RSS
├── screener.ts           # Screener filter types
├── platform.ts           # Domain manifest, feature flags, navigation
└── compute.ts            # PortfolioMetrics, RebalancingStrategy (display types)
```

> **Files removed from v2.0:** `api.ts`, `providers.ts`

---

## common.ts

```typescript
/** Discriminated union for error handling without exceptions. */
export type Result<T, E = Error> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E };

/** Progress event for long-running computations (received via Tauri events). */
export interface ProgressEvent {
  readonly phase: string;
  readonly current: number;
  readonly total: number;
  readonly message?: string;
}

/** ISO date string in YYYY-MM-DD format. */
export type ISODateString = string;

/** ISO datetime string in full ISO 8601 format. */
export type ISODateTimeString = string;

/** UUID string. */
export type UUID = string;
```

> **Removed from v2.0:** `Observable<T>` interface (was used for Comlink WebWorker progress streams; replaced by Tauri event listeners).

---

## compute.ts

```typescript
/** All output metrics from the backtest engine. */
export interface PortfolioMetrics {
  readonly totalReturn: number;
  readonly cagr: number;
  readonly annualizedVolatility: number;
  readonly sharpeRatio: number;
  readonly sortinoRatio: number;
  readonly maxDrawdown: number;
  readonly calmarRatio: number;
  readonly treynorRatio?: number;
  readonly alpha?: number;
  readonly beta?: number;
  readonly rSquared?: number;
  readonly informationRatio?: number;
  readonly skewness: number;
  readonly kurtosis: number;
  readonly bestYear: number;
  readonly worstYear: number;
  readonly var95: number;
  readonly cvar95: number;
  readonly winRate: number;
  readonly ulcerIndex: number;
}

/** Strategy interface for rebalancing extensibility. */
export interface RebalancingStrategy {
  readonly name: string;
  readonly description: string;
  readonly configSchema: unknown;
}

export interface StrategyContext {
  readonly currentDate: string;
  readonly currentWeights: Map<string, number>;
  readonly targetWeights: Map<string, number>;
  readonly portfolioValue: number;
  readonly daysSinceLastRebalance: number;
  readonly priceHistory: Map<string, readonly number[]>;
}
```

> **Removed from v2.0:** `ComputeService` interface (was the abstraction over WASM/TS fallback/server execution strategies). In the desktop app, the frontend calls Tauri commands directly via `invoke()`. The `ComputeService` abstraction is unnecessary when there's only one execution path: native Rust.

---

## All Other Files

The following files are **identical to v2.0** and are not reproduced here to avoid duplication. Refer to the v2.0 `type-definitions-reference.md` for their complete contents:

- `market-data.ts` — DailyPrice, AssetMetadata, RealTimeQuote, OptionsChain, FundamentalData, etc.
- `portfolio.ts` — PortfolioAllocation, PortfolioConfig, PortfolioContext, PortfolioType
- `backtest.ts` — BacktestConfig, BacktestResult, EquityCurvePoint, DrawdownPoint, etc.
- `monte-carlo.ts` — MonteCarloConfig, MonteCarloResult, HistogramBin (note: `maxMCIterations` in `@chrdfin/config` is now 1,000,000)
- `tracker.ts` — Holding, HoldingWithQuote, Transaction, Watchlist, PortfolioSummary, TransactionInput
- `optimizer.ts` — OptimizationConfig, OptimizationResult, EfficientFrontierConfig, EfficientFrontierResult, etc.
- `calculators.ts` — All calculator config/result types (compound growth, retirement, withdrawal, options payoff, risk/reward, position size)
- `news.ts` — NewsArticle, EarningsCalendarEntry, RSSFeedSource, NewsFilter
- `screener.ts` — ScreenerFilter, ScreenerConfig, ScreenerResult, ScreenerRow
- `platform.ts` — DomainManifest, NavigationItem, FeatureId, NavigationSection

These types are platform-agnostic — they describe data shapes, not transport mechanisms. They work identically whether the data comes from a REST API response or a Tauri `invoke()` result.
