# Data Fetching Patterns — chrdfin

## Purpose

Conventions for all data flow between the React frontend and the Rust backend in chrdfin. Covers cache key design, invalidation strategy, optimistic mutations, real-time event subscriptions via Tauri's event system, error handling, and per-domain query option presets.

This is the implementation contract for everything that calls a Tauri command. If a route or component fetches data, it follows the patterns here.

**Companion documents:**

- `docs/ui-component-recipes.md` — `useTauriQuery`, `useTauriMutation` hook implementations
- `docs/route-conventions.md` — search params drive query inputs; this doc covers what happens after
- `docs/technical-blueprint.md` — Tauri command catalog and Rust-side architecture
- `docs/database-schema-reference.md` — DuckDB schema (informs invalidation graph)
- `docs/type-definitions-reference.md` — shared types in `@chrdfin/types`

---

## Package Boundaries

| Concern | Location |
|---|---|
| `useTauriQuery`, `useTauriMutation` | `apps/desktop/src/hooks/use-tauri-command.ts` |
| `useTauriEvent` | `apps/desktop/src/hooks/use-tauri-event.ts` |
| `useProgress` | `apps/desktop/src/hooks/use-progress.ts` |
| Query key factory | `apps/desktop/src/lib/query-keys.ts` |
| QueryClient instance | `apps/desktop/src/App.tsx` (per `ui-component-recipes.md` section 10) |
| Domain-specific query hooks | Colocated with the domain that owns them, e.g. `apps/desktop/src/routes/tracking/hooks/use-portfolio.ts` |

---

## Stack

| Package | Use |
|---|---|
| `@tanstack/react-query` | Cache, deduplication, background refresh, optimistic mutations |
| `@tanstack/react-query-devtools` | Dev-only inspector |
| `@tauri-apps/api` | `invoke()` for commands, `listen()` for events |

QueryClient configured with conservative defaults — chrdfin is data-rich and most queries are deterministic given their inputs, so aggressive caching is appropriate.

```typescript
// apps/desktop/src/App.tsx
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      refetchOnWindowFocus: false,
      refetchOnReconnect: false,
      staleTime: 60 * 1000, // 1 minute
      gcTime: 30 * 60 * 1000, // 30 minutes
    },
    mutations: {
      retry: false,
    },
  },
});
```

**Defaults rationale:**

- `retry: false` — Tauri commands either succeed (cached and reused) or fail with a deterministic error (validation, missing data, computation error). Retrying transient I/O failures is a per-call decision, not a global default.
- `refetchOnWindowFocus: false` — Tauri webview "focus" doesn't carry the same staleness signal as a browser tab. Live data uses explicit subscriptions; static data doesn't need refresh-on-focus.
- `staleTime: 60_000` — most data is stable for at least a minute. Per-query overrides extend or shorten as needed.

---

## 1. Query Key Factory

Centralized, typed query keys via a factory. Avoids string drift across the codebase and makes cache invalidation auditable.

### File: `apps/desktop/src/lib/query-keys.ts`

```typescript
/**
 * Centralized query key factory.
 *
 * Every TanStack Query call in the application uses keys constructed
 * from this factory. Inline arrays like `["portfolio", id]` are
 * forbidden — they cause silent invalidation bugs when one site
 * changes the key shape and another doesn't.
 *
 * Naming convention:
 *   - Top-level domain (portfolios, holdings, quotes, news, etc.)
 *   - Optional sub-resource (lists, details, stats)
 *   - Optional input parameters
 *
 * Each domain exports an `all` key for full-domain invalidation:
 *   queryClient.invalidateQueries({ queryKey: keys.portfolios.all });
 */

import type {
  BacktestConfig,
  MonteCarloConfig,
  ScreenerFilters,
} from "@chrdfin/types";

export const keys = {
  /* ---------- Portfolios & holdings ---------- */
  portfolios: {
    all: ["portfolios"] as const,
    list: () => [...keys.portfolios.all, "list"] as const,
    detail: (id: string) => [...keys.portfolios.all, "detail", id] as const,
  },
  holdings: {
    all: ["holdings"] as const,
    byPortfolio: (portfolioId: string) =>
      [...keys.holdings.all, "by-portfolio", portfolioId] as const,
  },
  transactions: {
    all: ["transactions"] as const,
    byPortfolio: (portfolioId: string) =>
      [...keys.transactions.all, "by-portfolio", portfolioId] as const,
  },
  watchlists: {
    all: ["watchlists"] as const,
    list: () => [...keys.watchlists.all, "list"] as const,
    detail: (id: string) => [...keys.watchlists.all, "detail", id] as const,
  },

  /* ---------- Market data ---------- */
  quotes: {
    all: ["quotes"] as const,
    bySymbol: (symbol: string) => [...keys.quotes.all, symbol] as const,
    bySymbols: (symbols: ReadonlyArray<string>) =>
      [...keys.quotes.all, "batch", [...symbols].sort().join(",")] as const,
  },
  prices: {
    all: ["prices"] as const,
    history: (symbol: string, from?: string, to?: string) =>
      [...keys.prices.all, "history", symbol, from ?? "", to ?? ""] as const,
  },
  tickerMeta: {
    all: ["ticker-meta"] as const,
    bySymbol: (symbol: string) => [...keys.tickerMeta.all, symbol] as const,
    search: (query: string) =>
      [...keys.tickerMeta.all, "search", query] as const,
  },
  options: {
    all: ["options"] as const,
    chain: (symbol: string, expiry?: string) =>
      [...keys.options.all, "chain", symbol, expiry ?? ""] as const,
  },

  /* ---------- News & calendar ---------- */
  news: {
    all: ["news"] as const,
    list: (params: Record<string, unknown>) =>
      [...keys.news.all, "list", params] as const,
    detail: (id: string) => [...keys.news.all, "detail", id] as const,
  },
  calendar: {
    all: ["calendar"] as const,
    earnings: (from: string, to: string) =>
      [...keys.calendar.all, "earnings", from, to] as const,
    economic: (from: string, to: string) =>
      [...keys.calendar.all, "economic", from, to] as const,
  },

  /* ---------- Macro / FRED ---------- */
  macro: {
    all: ["macro"] as const,
    series: (seriesId: string) =>
      [...keys.macro.all, "series", seriesId] as const,
  },

  /* ---------- Computation results ---------- */
  backtest: {
    all: ["backtest"] as const,
    result: (config: BacktestConfig) =>
      [...keys.backtest.all, "result", config] as const,
  },
  monteCarlo: {
    all: ["monte-carlo"] as const,
    result: (config: MonteCarloConfig) =>
      [...keys.monteCarlo.all, "result", config] as const,
  },
  screener: {
    all: ["screener"] as const,
    results: (filters: ScreenerFilters) =>
      [...keys.screener.all, "results", filters] as const,
  },

  /* ---------- System ---------- */
  settings: {
    all: ["settings"] as const,
    bySetting: (key: string) =>
      [...keys.settings.all, key] as const,
  },
  syncStatus: {
    all: ["sync-status"] as const,
    current: () => [...keys.syncStatus.all, "current"] as const,
  },
} as const;
```

**Tradeoffs:**

- Object-shaped query inputs (e.g. `BacktestConfig`) are passed as the last segment. TanStack Query does deep equality on keys, so two calls with structurally equivalent configs share a cache entry. Be careful not to include unstable references (functions, dates as `Date` objects) — use serializable shapes only.
- Symbol lists in batched quote keys are sorted before joining so `["AAPL","MSFT"]` and `["MSFT","AAPL"]` hit the same cache entry. The component-side hook is responsible for matching this convention; pass sorted arrays in.
- The `as const` assertions preserve literal types through the factory so TanStack Query's type system can match keys at compile time.

---

## 2. Query Options by Data Category

Each data category has a recommended preset. Using these consistently is what makes invalidation predictable.

### File: `apps/desktop/src/lib/query-presets.ts`

```typescript
import type { UseQueryOptions } from "@tanstack/react-query";

type Preset<TResult> = Pick<
  UseQueryOptions<TResult, Error>,
  "staleTime" | "gcTime" | "refetchInterval" | "refetchIntervalInBackground"
>;

const MINUTE = 60 * 1000;
const HOUR = 60 * MINUTE;
const DAY = 24 * HOUR;

/* ---------- Static reference data ---------- */

/**
 * Ticker metadata, asset details — change rarely (corporate actions).
 * Cache for a day; survive reload.
 */
export const REFERENCE_DATA: Preset<unknown> = {
  staleTime: 1 * DAY,
  gcTime: 7 * DAY,
};

/* ---------- Historical data ---------- */

/**
 * Daily prices, dividends, macro series — historical data is immutable
 * within the trading day. Cache long; invalidate on sync completion.
 */
export const HISTORICAL_DATA: Preset<unknown> = {
  staleTime: 1 * HOUR,
  gcTime: 1 * DAY,
};

/* ---------- Computed results ---------- */

/**
 * Backtest, Monte Carlo, optimizer results — deterministic given inputs.
 * Cache forever; never refetch automatically.
 */
export const COMPUTED_RESULT: Preset<unknown> = {
  staleTime: Infinity,
  gcTime: Infinity,
};

/* ---------- Live quotes (without subscription) ---------- */

/**
 * For routes that show a quote but don't need tick-by-tick updates
 * (e.g. ticker detail page header). Polls every 5s while focused.
 *
 * For high-frequency updates (options chain real-time), prefer
 * useTauriEvent subscription instead.
 */
export const LIVE_QUOTE_POLL: Preset<unknown> = {
  staleTime: 0,
  gcTime: 5 * MINUTE,
  refetchInterval: 5 * 1000,
  refetchIntervalInBackground: false,
};

/* ---------- News & feed data ---------- */

/**
 * News feeds — moderate refresh cadence.
 */
export const FEED_DATA: Preset<unknown> = {
  staleTime: 5 * MINUTE,
  gcTime: 30 * MINUTE,
  refetchInterval: 5 * MINUTE,
  refetchIntervalInBackground: false,
};

/* ---------- Portfolio state ---------- */

/**
 * Portfolio composition, holdings derived from transactions —
 * change only on user action (mutation invalidates).
 */
export const USER_DATA: Preset<unknown> = {
  staleTime: Infinity,
  gcTime: 1 * DAY,
};

/* ---------- Settings ---------- */

/**
 * Settings rarely change and changes are user-initiated (mutation
 * invalidates). Cache forever.
 */
export const SETTINGS: Preset<unknown> = {
  staleTime: Infinity,
  gcTime: Infinity,
};
```

### Application

```typescript
import { useTauriQuery } from "@/hooks/use-tauri-command";
import { keys } from "@/lib/query-keys";
import { COMPUTED_RESULT, USER_DATA, REFERENCE_DATA } from "@/lib/query-presets";

// Backtest result — cached forever
const { data } = useTauriQuery(
  "run_backtest",
  config,
  { queryKey: keys.backtest.result(config), ...COMPUTED_RESULT },
);

// Portfolio holdings — invalidated on transaction mutation
const { data } = useTauriQuery(
  "get_holdings",
  { portfolioId },
  { queryKey: keys.holdings.byPortfolio(portfolioId), ...USER_DATA },
);

// Ticker metadata — cached for a day
const { data } = useTauriQuery(
  "get_ticker_metadata",
  { symbol },
  { queryKey: keys.tickerMeta.bySymbol(symbol), ...REFERENCE_DATA },
);
```

**Note:** the `useTauriQuery` signature in `ui-component-recipes.md` derives the queryKey from `[command, args]` by default. To use the factory keys instead, pass an explicit `queryKey` option. Either approach works; use the factory for any query you want to invalidate from outside the component.

---

## 3. Invalidation Graph

Mutations invalidate downstream queries. The graph below documents which mutations affect which query keys. Update this table whenever a new mutation is added.

| Mutation command | Invalidates |
|---|---|
| `create_portfolio` | `keys.portfolios.all` |
| `update_portfolio` | `keys.portfolios.detail(id)`, `keys.portfolios.list()` |
| `delete_portfolio` | `keys.portfolios.all`, `keys.holdings.byPortfolio(id)`, `keys.transactions.byPortfolio(id)` |
| `add_transaction` | `keys.transactions.byPortfolio(id)`, `keys.holdings.byPortfolio(id)`, `keys.portfolios.detail(id)` |
| `update_transaction` | Same as `add_transaction` |
| `delete_transaction` | Same as `add_transaction` |
| `add_to_watchlist` | `keys.watchlists.detail(id)`, `keys.watchlists.list()` |
| `remove_from_watchlist` | Same as `add_to_watchlist` |
| `sync_data` | `keys.prices.all`, `keys.quotes.all`, `keys.tickerMeta.all`, `keys.macro.all`, `keys.news.all` |
| `sync_news` | `keys.news.all` |
| `set_setting` | `keys.settings.bySetting(key)` |
| `save_calculator_state` | (no invalidation — only affects deferred reads) |

### Mutation hook pattern

Domain-specific mutation hooks colocate the mutation call with its invalidation contract. This is what the per-domain hook directories are for.

#### File: `apps/desktop/src/routes/tracking/hooks/use-add-transaction.ts`

```typescript
import { useQueryClient } from "@tanstack/react-query";
import { useTauriMutation } from "@/hooks/use-tauri-command";
import { keys } from "@/lib/query-keys";
import type { TransactionInput, Transaction } from "@chrdfin/types";

/**
 * Add a transaction to a portfolio.
 *
 * Invalidates:
 *   - transactions.byPortfolio(portfolioId)
 *   - holdings.byPortfolio(portfolioId)
 *   - portfolios.detail(portfolioId)  (totals change)
 */
export function useAddTransaction(portfolioId: string) {
  const queryClient = useQueryClient();

  return useTauriMutation<Transaction, TransactionInput>("add_transaction", {
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: keys.transactions.byPortfolio(portfolioId),
      });
      queryClient.invalidateQueries({
        queryKey: keys.holdings.byPortfolio(portfolioId),
      });
      queryClient.invalidateQueries({
        queryKey: keys.portfolios.detail(portfolioId),
      });
    },
  });
}
```

**Tradeoffs:**

- Invalidation is explicit and per-mutation. Auto-invalidation via tag-based systems (e.g. RTK Query) is cleaner in some ways but the graph is small enough here that explicit invalidation is more auditable.
- Each mutation hook is per-portfolio (closes over `portfolioId`). Multi-portfolio bulk mutations would need a different pattern; chrdfin doesn't have any in scope through Phase 8.

---

## 4. Optimistic Mutations

For mutations whose result is predictable from the input (rename, watchlist add/remove, transaction edit), optimistic updates give an instant UI response. The pattern: snapshot, update, rollback on error.

### Pattern

```typescript
import { useQueryClient } from "@tanstack/react-query";
import { useTauriMutation } from "@/hooks/use-tauri-command";
import { keys } from "@/lib/query-keys";
import type { Watchlist } from "@chrdfin/types";

interface AddToWatchlistArgs {
  watchlistId: string;
  symbol: string;
}

export function useAddToWatchlist() {
  const queryClient = useQueryClient();

  return useTauriMutation<void, AddToWatchlistArgs>("add_to_watchlist", {
    onMutate: async ({ watchlistId, symbol }) => {
      // 1. Cancel any in-flight refetches that would overwrite our update.
      await queryClient.cancelQueries({
        queryKey: keys.watchlists.detail(watchlistId),
      });

      // 2. Snapshot the current state for rollback.
      const previous = queryClient.getQueryData<Watchlist>(
        keys.watchlists.detail(watchlistId),
      );

      // 3. Optimistically update.
      if (previous) {
        queryClient.setQueryData<Watchlist>(
          keys.watchlists.detail(watchlistId),
          {
            ...previous,
            symbols: [...previous.symbols, symbol],
          },
        );
      }

      // 4. Return context for rollback.
      return { previous };
    },

    onError: (_err, { watchlistId }, context) => {
      // Rollback to snapshot.
      if (context?.previous) {
        queryClient.setQueryData(
          keys.watchlists.detail(watchlistId),
          context.previous,
        );
      }
    },

    onSettled: ({ watchlistId }) => {
      // Refetch to reconcile with server truth (covers cases where the
      // optimistic update was correct but server adds derived data —
      // e.g. timestamps).
      queryClient.invalidateQueries({
        queryKey: keys.watchlists.detail(watchlistId),
      });
    },
  });
}
```

### When to use optimistic vs invalidation-only

| Decision | Use |
|---|---|
| Mutation effect is local to a single object (rename, toggle, add to list) | Optimistic |
| Mutation triggers complex recomputation (transaction → holdings → portfolio totals) | Invalidation only — let the query refetch |
| Mutation latency is < 50ms (fast Rust commands) | Invalidation only — optimism saves no perceived time |
| Mutation might fail due to validation or backend constraint | Optimistic carefully — rollback path must be solid |

**For chrdfin specifically:**

- Transactions: invalidation only (recomputes holdings)
- Watchlist add/remove: optimistic (single-array mutation)
- Portfolio rename: optimistic
- Settings: invalidation only (rarely user-blocking)
- Calculator save: invalidation only

---

## 5. Tauri Events: Real-Time Subscriptions

Tauri's event system is the right pattern for high-frequency or push-driven updates: live quotes, sync progress, computation progress, OS-level notifications.

### Event naming convention

Events are namespaced by domain and use `://` as separator (Tauri convention):

| Event name | Payload | Emitted by |
|---|---|---|
| `quote://update` | `QuoteUpdate` | Polygon polling loop in `quotes.rs` |
| `sync://progress` | `SyncProgress` | Sync engine in `sync/*.rs` |
| `sync://complete` | `SyncComplete` | Sync engine on finish |
| `compute://progress` | `ComputeProgress` | Backtest/MC engines for long-running jobs |
| `compute://complete` | `ComputeComplete` | Computation finish |
| `news://new` | `NewsArticle` | RSS/news poller |

Per-symbol scoping is via payload, not event name (avoids unbounded event registration).

### Payload types in `@chrdfin/types`

```typescript
// packages/@chrdfin/types/src/events.ts

export interface QuoteUpdate {
  symbol: string;
  price: number;
  bid?: number;
  ask?: number;
  /** Unix milliseconds */
  timestamp: number;
  /** Volume in current session */
  volume: number;
  /** Day change as percent */
  dayChange: number;
}

export interface SyncProgress {
  /** Domain being synced: "prices" | "news" | "macro" | "ticker-meta" */
  domain: string;
  current: number;
  total: number;
  /** Optional human-readable status, e.g. "Fetching SPY history" */
  status?: string;
}

export interface SyncComplete {
  domain: string;
  /** Number of records updated */
  count: number;
  /** Duration in milliseconds */
  durationMs: number;
  /** Optional error if sync partially failed */
  error?: string;
}

export interface ComputeProgress {
  /** Computation ID (UUID generated by Rust on job start) */
  jobId: string;
  /** "backtest" | "monte-carlo" | "optimizer" */
  kind: string;
  current: number;
  total: number;
  status?: string;
}

export interface ComputeComplete {
  jobId: string;
  kind: string;
  durationMs: number;
  error?: string;
}
```

---

## 6. `useTauriEvent` Hook

Type-safe wrapper over Tauri's `listen()` API. Cleans up subscriptions on unmount.

### File: `apps/desktop/src/hooks/use-tauri-event.ts`

```typescript
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";

/**
 * Subscribe to a Tauri event.
 *
 * The handler is held in a ref so that consumers can pass an inline
 * function without re-subscribing on every render. The subscription
 * itself only re-registers if `event` changes.
 *
 * @example
 *   useTauriEvent<QuoteUpdate>("quote://update", (payload) => {
 *     if (payload.symbol === activeSymbol) {
 *       setLatestPrice(payload.price);
 *     }
 *   });
 */
export function useTauriEvent<TPayload>(
  event: string,
  handler: (payload: TPayload) => void,
  /** Set false to disable temporarily without unmounting. */
  enabled: boolean = true,
): void {
  const handlerRef = useRef(handler);

  // Keep the latest handler in a ref so we don't re-subscribe.
  useEffect(() => {
    handlerRef.current = handler;
  }, [handler]);

  useEffect(() => {
    if (!enabled) return;

    let unlisten: UnlistenFn | null = null;
    let cancelled = false;

    listen<TPayload>(event, (e) => {
      handlerRef.current(e.payload);
    }).then((fn) => {
      if (cancelled) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, [event, enabled]);
}
```

**Tradeoffs:**

- The `cancelled` flag handles the race between an early unmount and the async `listen()` resolution. Without it, a fast unmount-remount would leave a dangling subscription.
- The handler ref pattern lets consumers pass `(payload) => doSomething(payload)` inline without causing resubscription on every render. Tradeoff: the handler is always the latest, so closure-captured values can become stale — pass IDs/keys explicitly via the payload or via consistent props.
- `enabled` is checked once per effect run. To toggle dynamically without unmounting the component, the consumer can set `enabled: isActive` and the effect tears down/re-establishes appropriately.

---

## 7. Integrating Events with Query Cache

The most common pattern: an event fires, you update or invalidate a query.

### Pattern A: Direct cache update (low-frequency, predictable payload)

```typescript
import { useQueryClient } from "@tanstack/react-query";
import { useTauriEvent } from "@/hooks/use-tauri-event";
import { keys } from "@/lib/query-keys";
import type { QuoteUpdate } from "@chrdfin/types";

/**
 * Subscribe to live quotes for a single symbol and write updates
 * directly into the query cache.
 *
 * Use this when:
 *   - You have a current cache entry for the quote
 *   - The event payload is the complete updated quote
 *   - You want components reading from useTauriQuery(quotes...) to
 *     receive the update without an extra refetch
 */
export function useLiveQuoteSubscription(symbol: string): void {
  const queryClient = useQueryClient();

  useTauriEvent<QuoteUpdate>(
    "quote://update",
    (payload) => {
      if (payload.symbol !== symbol) return;
      queryClient.setQueryData(keys.quotes.bySymbol(symbol), payload);
    },
    !!symbol,
  );
}
```

### Pattern B: Invalidation (event indicates "things changed")

```typescript
/**
 * Invalidate price queries when a sync completes successfully.
 */
export function useSyncCompleteListener(): void {
  const queryClient = useQueryClient();

  useTauriEvent<SyncComplete>("sync://complete", (payload) => {
    if (payload.error) return; // Don't invalidate on partial failure

    if (payload.domain === "prices") {
      queryClient.invalidateQueries({ queryKey: keys.prices.all });
      queryClient.invalidateQueries({ queryKey: keys.holdings.all });
    } else if (payload.domain === "news") {
      queryClient.invalidateQueries({ queryKey: keys.news.all });
    } else if (payload.domain === "macro") {
      queryClient.invalidateQueries({ queryKey: keys.macro.all });
    }
  });
}
```

### Pattern C: Local state (transient progress, not queryable)

```typescript
/**
 * Track real-time progress of a computation job. Not cached — once
 * the job is done, the "progress" no longer exists.
 */
export function useComputeProgress(jobId: string | null): {
  current: number;
  total: number;
  status: string;
} {
  const [progress, setProgress] = useState({
    current: 0,
    total: 0,
    status: "",
  });

  useTauriEvent<ComputeProgress>(
    "compute://progress",
    (payload) => {
      if (payload.jobId !== jobId) return;
      setProgress({
        current: payload.current,
        total: payload.total,
        status: payload.status ?? "",
      });
    },
    jobId !== null,
  );

  return progress;
}
```

---

## 8. The `useProgress` Hook

The standard progress UI pattern for long-running Tauri commands. Combines `useTauriEvent` and local state into a reusable component-friendly shape.

### File: `apps/desktop/src/hooks/use-progress.ts`

```typescript
import { useState } from "react";
import { useTauriEvent } from "./use-tauri-event";
import type { ComputeProgress, ComputeComplete } from "@chrdfin/types";

export interface ProgressState {
  active: boolean;
  current: number;
  total: number;
  status: string;
  /** 0-1 fractional progress; null when no progress yet */
  fraction: number | null;
  /** Set when a job completes with an error */
  error: string | null;
}

const INITIAL: ProgressState = {
  active: false,
  current: 0,
  total: 0,
  status: "",
  fraction: null,
  error: null,
};

/**
 * Subscribe to progress events for a specific compute job.
 *
 * @param jobId — UUID returned by the Tauri command on job start, or
 *                null when no job is active
 *
 * @example
 *   const { mutateAsync } = useTauriMutation<{ jobId: string }, BacktestConfig>(
 *     "start_backtest",
 *   );
 *   const [jobId, setJobId] = useState<string | null>(null);
 *   const progress = useProgress(jobId);
 *
 *   const handleRun = async () => {
 *     const { jobId } = await mutateAsync(config);
 *     setJobId(jobId);
 *   };
 */
export function useProgress(jobId: string | null): ProgressState {
  const [state, setState] = useState<ProgressState>(INITIAL);

  useTauriEvent<ComputeProgress>(
    "compute://progress",
    (payload) => {
      if (payload.jobId !== jobId) return;
      setState({
        active: true,
        current: payload.current,
        total: payload.total,
        status: payload.status ?? "",
        fraction: payload.total > 0 ? payload.current / payload.total : null,
        error: null,
      });
    },
    jobId !== null,
  );

  useTauriEvent<ComputeComplete>(
    "compute://complete",
    (payload) => {
      if (payload.jobId !== jobId) return;
      setState((prev) => ({
        ...prev,
        active: false,
        fraction: 1,
        error: payload.error ?? null,
      }));
    },
    jobId !== null,
  );

  return state;
}
```

**Tradeoffs:**

- Two separate `useTauriEvent` calls instead of merging into one. Each event has a different payload shape; merging would require runtime discrimination. Two effects are clearer.
- State is per-component. If two components both want to track the same job, they each subscribe — Tauri events are broadcast, so each subscription receives all events. CPU cost is negligible at chrdfin's event volume.
- Final progress is set to `fraction: 1` on completion regardless of `current`/`total` — backends sometimes emit `complete` without a final progress update.

---

## 9. Bulk / Batched Fetches

Multiple symbols at once: prefer one round-trip with a batch command over N round-trips with single commands.

### Tauri command convention

The Rust side exposes paired `get_x` (single) and `get_xs_batch` (multi) commands:

```rust
#[tauri::command]
pub async fn get_quote(symbol: String) -> Result<Quote, String> { /* ... */ }

#[tauri::command]
pub async fn get_quotes_batch(symbols: Vec<String>) -> Result<Vec<Quote>, String> { /* ... */ }
```

### Frontend usage

```typescript
// Single symbol — direct
const { data: quote } = useTauriQuery(
  "get_quote",
  { symbol: "AAPL" },
  { queryKey: keys.quotes.bySymbol("AAPL"), ...LIVE_QUOTE_POLL },
);

// Multiple symbols — batch with sorted key
const symbols = ["AAPL", "MSFT", "GOOGL"];
const { data: quotes } = useTauriQuery(
  "get_quotes_batch",
  { symbols },
  { queryKey: keys.quotes.bySymbols(symbols), ...LIVE_QUOTE_POLL },
);
```

### Hydrating individual entries from a batch

When a batch fetch completes, populate the per-symbol cache so individual `useTauriQuery(quotes.bySymbol(...))` callers benefit too:

```typescript
import { useQueryClient } from "@tanstack/react-query";

function useBatchQuotes(symbols: ReadonlyArray<string>) {
  const queryClient = useQueryClient();

  return useTauriQuery(
    "get_quotes_batch",
    { symbols },
    {
      queryKey: keys.quotes.bySymbols(symbols),
      ...LIVE_QUOTE_POLL,
      // Hydrate per-symbol cache on each successful fetch.
      // Use 'select' to avoid an extra effect.
      select: (data: Quote[]) => {
        for (const quote of data) {
          queryClient.setQueryData(
            keys.quotes.bySymbol(quote.symbol),
            quote,
          );
        }
        return data;
      },
    },
  );
}
```

**Note:** Mutating the cache from `select` is unconventional; the documented alternative is `onSuccess` (deprecated in TanStack Query v5) or a dedicated `useEffect`. Either works. The `select` approach is concise but couples cache hydration to the active subscriber — once the screener row scrolls off, the per-symbol entries stop being kept fresh.

---

## 10. Error Handling

Errors from Tauri commands surface as JavaScript `Error` instances via the `useTauriCommand` wrapper.

### Per-query error handling

```typescript
const { data, error, isLoading } = useTauriQuery<Quote>(
  "get_quote",
  { symbol: "AAPL" },
);

if (error) {
  // error.message is the string returned by the Rust Result::Err arm,
  // translated by the thiserror-derived backend.
  return <span className="text-destructive">{error.message}</span>;
}
```

### Retry policy

| Scenario | Retry |
|---|---|
| Validation error from Rust (e.g. invalid ticker format) | Never — input is bad, retry is wasted |
| External API rate limit (Tiingo, Polygon, FRED) | Up to 3 with exponential backoff (handled in Rust adapter, not in TanStack Query) |
| DuckDB transient lock | Once after 100ms (handled in Rust) |
| Network failure during sync | Retry handled by Rust sync engine, not by frontend |
| Computation error (NaN propagation, singular matrix in optimization) | Never — surface to user with context |

**Principle:** retry policy lives in the Rust backend. The frontend trusts that what comes back is final. Frontend `retry: false` is the global default for this reason — TanStack Query retrying a Tauri command after a known-bad input would just produce identical errors.

### Mutation error UX

```typescript
const { mutate, isError, error, reset } = useAddTransaction(portfolioId);

return (
  <form onSubmit={handleSubmit((values) => mutate(values))}>
    {/* ... fields ... */}
    {isError ? (
      <div className="text-xs text-destructive">
        {error.message}
        <button onClick={reset}>Dismiss</button>
      </div>
    ) : null}
    <Button type="submit">Add Transaction</Button>
  </form>
);
```

`reset()` clears the error so the user can retry without remounting the form.

---

## 11. Background Refresh & Sync Coordination

When the user opens the app or hits "sync now," the data sync engine fires events that drive cache invalidation. The pattern coordinates the two systems.

### Sync invalidation hook (mounted once at root)

```typescript
// apps/desktop/src/components/providers/sync-provider.tsx
import { useQueryClient } from "@tanstack/react-query";
import { useTauriEvent } from "@/hooks/use-tauri-event";
import { keys } from "@/lib/query-keys";
import type { SyncComplete } from "@chrdfin/types";

const DOMAIN_TO_KEYS: Record<string, ReadonlyArray<readonly string[]>> = {
  prices: [keys.prices.all, keys.holdings.all, keys.portfolios.all],
  quotes: [keys.quotes.all],
  "ticker-meta": [keys.tickerMeta.all],
  macro: [keys.macro.all],
  news: [keys.news.all],
};

export function SyncProvider({ children }: { children: React.ReactNode }): JSX.Element {
  const queryClient = useQueryClient();

  useTauriEvent<SyncComplete>("sync://complete", (payload) => {
    if (payload.error) return;
    const keysToInvalidate = DOMAIN_TO_KEYS[payload.domain] ?? [];
    for (const key of keysToInvalidate) {
      queryClient.invalidateQueries({ queryKey: key });
    }
  });

  return <>{children}</>;
}
```

Mount inside `<QueryClientProvider>` in `App.tsx`:

```typescript
<QueryClientProvider client={queryClient}>
  <SyncProvider>
    <ThemeProvider>
      <RouterProvider router={router} />
    </ThemeProvider>
  </SyncProvider>
</QueryClientProvider>
```

### Sync status indicator

The platform shell header can show a sync indicator using a query against the sync state:

```typescript
const { data: syncStatus } = useTauriQuery<SyncStatus>(
  "get_sync_status",
  {},
  {
    queryKey: keys.syncStatus.current(),
    refetchInterval: 2000,
    refetchIntervalInBackground: false,
  },
);
```

The Rust side maintains a single sync state object readable at any time; the indicator polls every 2s.

---

## 12. Reference Implementations

### A. Live ticker quote on the ticker detail page

```typescript
// apps/desktop/src/routes/market/ticker/$symbol.lazy.tsx
export const Route = createLazyFileRoute("/market/ticker/$symbol")({
  component: TickerDetailPage,
});

function TickerDetailPage(): JSX.Element {
  const { symbol } = Route.useParams();
  const queryClient = useQueryClient();

  // Initial load — once.
  const { data: quote, error } = useTauriQuery<Quote>(
    "get_quote",
    { symbol },
    {
      queryKey: keys.quotes.bySymbol(symbol),
      staleTime: 5 * 1000,
    },
  );

  // Subscribe to live updates while this page is mounted.
  useTauriEvent<QuoteUpdate>("quote://update", (payload) => {
    if (payload.symbol !== symbol) return;
    queryClient.setQueryData<Quote>(keys.quotes.bySymbol(symbol), {
      ...payload,
      // Backend sends partial; merge with existing for fields not in payload
    });
  });

  if (error) return <ErrorState message={error.message} />;
  if (!quote) return <RoutePending />;

  return <TickerHeader quote={quote} />;
}
```

### B. Backtest with progress

```typescript
function BacktestPage(): JSX.Element {
  const search = Route.useSearch();
  const [jobId, setJobId] = useState<string | null>(null);
  const progress = useProgress(jobId);

  const startBacktest = useTauriMutation<{ jobId: string }, BacktestConfig>(
    "start_backtest",
  );

  const { data: result } = useTauriQuery<BacktestResult>(
    "get_backtest_result",
    { jobId },
    {
      queryKey: keys.backtest.result(search),
      enabled: jobId !== null && !progress.active && !progress.error,
      ...COMPUTED_RESULT,
    },
  );

  const handleRun = async (config: BacktestConfig) => {
    const { jobId } = await startBacktest.mutateAsync(config);
    setJobId(jobId);
  };

  if (progress.active) {
    return (
      <ProgressBar
        fraction={progress.fraction ?? 0}
        status={progress.status}
      />
    );
  }

  if (progress.error) {
    return <ErrorState message={progress.error} onRetry={() => handleRun(config)} />;
  }

  return result ? <BacktestResults data={result} /> : <BacktestForm onRun={handleRun} />;
}
```

### C. Optimistic watchlist add

```typescript
function WatchlistPage(): JSX.Element {
  const { data: watchlist } = useTauriQuery<Watchlist>(
    "get_watchlist",
    { id: "default" },
    { queryKey: keys.watchlists.detail("default"), ...USER_DATA },
  );

  const addToWatchlist = useAddToWatchlist(); // optimistic mutation hook from section 4

  const handleAdd = (symbol: string) => {
    addToWatchlist.mutate({ watchlistId: "default", symbol });
  };

  return (
    <div>
      <SymbolInput onAdd={handleAdd} />
      {watchlist?.symbols.map((s) => <WatchlistRow key={s} symbol={s} />)}
    </div>
  );
}
```

---

## 13. Common Pitfalls

| Symptom | Cause | Fix |
|---|---|---|
| Query refetches on every render | Object/array reference in `args` changing each render | Memoize the args object, or pass primitives only |
| Cache miss between two components asking for the same data | Inline string keys differ (e.g. `"AAPL"` vs `" AAPL "`) | Use the `keys` factory exclusively; normalize inputs (trim, uppercase) before passing |
| Optimistic update flickers back | `onSettled` invalidate races with a slow refetch | Add a small `setTimeout(invalidate, 200)` if mutation latency is variable, or skip the invalidate when payload contains all derived fields |
| Tauri event fires but handler doesn't run | Subscription set up before event listener is mounted on Rust side | Tauri events are buffered — this is rarely the cause. More likely: handler set on wrong event name (typo `quote://updates`) |
| Memory grows over a long session | `gcTime: Infinity` on rapidly-changing keys (e.g. quotes per symbol) | Use `gcTime: 5 * MINUTE` for quote data; only computed results deserve infinite GC |
| Mutation rolls back even though server succeeded | `onError` triggered by a downstream error in `onSuccess` callback | Move heavy logic out of `onSuccess`; only invalidate inside it |
| `useTauriEvent` resubscribes every render | Handler not in a ref | Verify the hook implementation includes the handler ref pattern |
| Batch query results in N+1 individual queries | Component using `useQueries` with one entry per symbol | Use `get_xs_batch` Tauri command + single `useTauriQuery`, then hydrate individual entries via `select` or `setQueryData` |
| Stale data after `sync://complete` | `SyncProvider` not mounted, or domain string mismatch | Verify provider is in the React tree above all consumers; check that Rust emits exactly the expected domain string |
| QueryClient persists across HMR with stale callbacks | New code running with old cache state | Hard-refresh during dev (Cmd/Ctrl+R inside webview) — this is fine in development, never an issue in production |
| Long-running computation completes while user navigates away | Component unmounts, progress hook cleanup runs, but the result query hasn't been triggered yet | Store `jobId` in a global query (e.g. a `keys.jobs.byId(jobId)` cache entry) so it survives unmount; or rehydrate on mount via `get_job_status` |

---

## 14. Testing Patterns

### Mocking Tauri commands in unit tests

```typescript
// vitest setup file
import { vi } from "vitest";

const invokeMock = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
}));

beforeEach(() => {
  invokeMock.mockReset();
});

// In tests:
test("portfolio query loads", async () => {
  invokeMock.mockResolvedValueOnce({
    id: "abc",
    name: "Test",
    symbols: ["AAPL"],
  });

  // Render component, assert it shows data
});
```

### Mocking Tauri events

```typescript
// Provide a manual emitter
const listeners = new Map<string, Array<(p: unknown) => void>>();

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(async (event: string, handler: (e: { payload: unknown }) => void) => {
    const wrapped = (p: unknown) => handler({ payload: p } as never);
    const arr = listeners.get(event) ?? [];
    arr.push(wrapped);
    listeners.set(event, arr);
    return () => {
      const updated = (listeners.get(event) ?? []).filter((fn) => fn !== wrapped);
      listeners.set(event, updated);
    };
  }),
}));

export function emitTestEvent(event: string, payload: unknown): void {
  for (const fn of listeners.get(event) ?? []) fn(payload);
}
```

In tests:

```typescript
test("live quote subscription updates cache", async () => {
  // ... render component subscribing to quote://update for AAPL
  emitTestEvent("quote://update", {
    symbol: "AAPL",
    price: 195.50,
    timestamp: Date.now(),
    volume: 1_000_000,
    dayChange: 1.2,
  });

  await waitFor(() => {
    expect(screen.getByText(/195\.50/)).toBeInTheDocument();
  });
});
```

### Tolerance for cache races

Cache-related tests are inherently async. Use `waitFor` with reasonable timeouts; never assert state synchronously after a mutation or event emission.

---

## 15. References

- TanStack Query docs: <https://tanstack.com/query/latest>
- TanStack Query optimistic updates: <https://tanstack.com/query/latest/docs/framework/react/guides/optimistic-updates>
- Tauri command docs: <https://v2.tauri.app/develop/calling-rust/>
- Tauri event docs: <https://v2.tauri.app/develop/calling-frontend/>
- Query key conventions: <https://tkdodo.eu/blog/effective-react-query-keys>

---

## 16. Document Maintenance

When adding a new Tauri command:

1. Add an entry to the query-keys factory if the command's results are cacheable.
2. Add a row to the invalidation graph table (section 3) listing which query keys this mutation invalidates.
3. Choose a query preset from `query-presets.ts` and apply it in the consuming hook.
4. Document any new event payload types in `@chrdfin/types/src/events.ts`.

When adding a new event:

1. Document the event name and payload in section 5.
2. Update the `DOMAIN_TO_KEYS` map in `SyncProvider` if the event implies cache invalidation.
3. Define the payload type in `@chrdfin/types/src/events.ts`.

When changing a query key shape:

1. Audit every consumer (search the `keys.<domain>` factory for usages).
2. Update both the factory and all consumers in the same commit.
3. Run `pnpm typecheck` — most key shape errors surface as type errors due to literal types.
