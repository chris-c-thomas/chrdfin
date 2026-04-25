# Route Conventions — chrdfin

## Purpose

Conventions for client-side routing in chrdfin's Tauri SPA. Covers the file-based route tree, search param schemas, type-safe navigation, error and pending boundaries, code-splitting strategy, and feature flag integration.

This file is the implementation contract for everything in `apps/desktop/src/routes`. Every route file in the application either follows the patterns below or documents why it deviates.

**Companion documents:**
- `docs/ui-design-system.md` — visual tokens, density, platform shell
- `docs/ui-component-recipes.md` — `<ThemeProvider>`, `useTauriCommand`, hooks
- `docs/chart-recipes.md` — chart components used inside route pages
- `docs/technical-blueprint.md` — system architecture
- `docs/type-definitions-reference.md` — shared types and Zod schemas (search param schemas live here)

---

## Package Boundaries

| Concern | Location |
|---|---|
| Route definitions | `apps/desktop/src/routes/**` |
| Generated route tree | `apps/desktop/src/routeTree.gen.ts` (auto, gitignored) |
| Router instance | `apps/desktop/src/router.tsx` |
| Search param schemas | `packages/@chrdfin/types/src/search-params.ts` |
| Domain-specific navigation hooks | `apps/desktop/src/hooks/use-domain-nav.ts` (per domain) |
| Section error/pending components | `apps/desktop/src/components/shell/route-states.tsx` |

---

## Routing Stack

| Package | Version | Notes |
|---|---|---|
| `@tanstack/react-router` | latest | Type-safe client routing |
| `@tanstack/router-plugin` | latest | Vite plugin, generates `routeTree.gen.ts` |
| `@tanstack/router-devtools` | latest | Dev-only inspector (excluded from production bundle) |
| `zod` | ^3 | Search param validation |

### Vite plugin configuration

```typescript
// apps/desktop/vite.config.ts
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import { TanStackRouterVite } from "@tanstack/router-plugin/vite";
import path from "node:path";

export default defineConfig({
  plugins: [
    TanStackRouterVite({
      routesDirectory: "./src/routes",
      generatedRouteTree: "./src/routeTree.gen.ts",
      routeFileIgnorePrefix: "-",
      quoteStyle: "double",
    }),
    react(),
  ],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Tauri-specific
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  envPrefix: ["VITE_", "TAURI_"],
  build: {
    target: "es2022",
    minify: "esbuild",
    sourcemap: true,
  },
});
```

The plugin watches `./src/routes` and regenerates `routeTree.gen.ts` on every file change. The generated file is **never edited by hand** and is added to `.gitignore`. It is regenerated on `pnpm dev` and on CI before typecheck.

---

## File Layout

The complete Phase 0 route tree. Every file listed below is created in Phase 0 with at minimum a placeholder component; later phases swap in real implementations.

```
apps/desktop/src/routes/
├── __root.tsx                        # Platform shell, root error boundary
├── index.tsx                         # / → home (redirects to last-active section)
│
├── analysis/
│   ├── route.tsx                     # /analysis layout + section error boundary
│   ├── index.tsx                     # /analysis → redirects to /analysis/backtest
│   ├── backtest.tsx                  # /analysis/backtest (search params)
│   ├── backtest.lazy.tsx             # Lazy chunk for backtest component
│   ├── monte-carlo.tsx               # /analysis/monte-carlo (search params)
│   ├── monte-carlo.lazy.tsx
│   ├── optimizer.tsx                 # /analysis/optimizer (Phase 9, gated)
│   └── optimizer.lazy.tsx
│
├── tracking/
│   ├── route.tsx
│   ├── index.tsx                     # /tracking → /tracking/portfolio
│   ├── portfolio.tsx                 # /tracking/portfolio (search: portfolioId)
│   ├── portfolio.lazy.tsx
│   ├── transactions.tsx              # /tracking/transactions
│   ├── transactions.lazy.tsx
│   ├── watchlist.tsx                 # /tracking/watchlist
│   └── watchlist.lazy.tsx
│
├── tools/
│   ├── route.tsx
│   ├── index.tsx                     # /tools → /tools/calculators
│   ├── calculators/
│   │   ├── route.tsx
│   │   ├── index.tsx                 # /tools/calculators (calculator picker)
│   │   ├── compound-interest.tsx     # /tools/calculators/compound-interest
│   │   ├── compound-interest.lazy.tsx
│   │   ├── retirement.tsx
│   │   ├── retirement.lazy.tsx
│   │   ├── savings-rate.tsx
│   │   └── savings-rate.lazy.tsx
│   ├── compare.tsx                   # /tools/compare
│   └── compare.lazy.tsx
│
├── market/
│   ├── route.tsx
│   ├── index.tsx                     # /market → /market/screener
│   ├── screener.tsx                  # /market/screener (search: filters)
│   ├── screener.lazy.tsx
│   ├── ticker/
│   │   ├── $symbol.tsx               # /market/ticker/$symbol
│   │   └── $symbol.lazy.tsx
│   ├── options/
│   │   ├── $symbol.tsx               # /market/options/$symbol
│   │   └── $symbol.lazy.tsx
│   ├── news.tsx                      # /market/news (search: category, source)
│   ├── news.lazy.tsx
│   ├── calendar.tsx                  # /market/calendar (search: date range)
│   └── calendar.lazy.tsx
│
└── -shared/                          # Ignored by router (prefix "-")
    ├── route-states.tsx              # PendingComponent, ErrorComponent
    └── feature-gate.tsx              # <FeatureGate /> wrapper
```

### File naming conventions

| Pattern | Meaning |
|---|---|
| `__root.tsx` | Root layout, always loaded |
| `index.tsx` | Index of the parent path (e.g. `analysis/index.tsx` → `/analysis`) |
| `route.tsx` | Layout route at the parent path (e.g. `analysis/route.tsx` → `/analysis` parent route, wraps children) |
| `$param.tsx` | Dynamic path parameter (e.g. `ticker/$symbol.tsx` → `/ticker/AAPL`) |
| `*.lazy.tsx` | Lazy-loaded component (route metadata in `*.tsx`, component in `*.lazy.tsx`) |
| `-prefix/` | Ignored by router; used for utility files inside `routes/` |

---

## 1. Router Instance

Single router instance constructed in `router.tsx`, mounted by `App.tsx` (per `ui-component-recipes.md` section 10).

### File: `apps/desktop/src/router.tsx`

```typescript
import { createRouter } from "@tanstack/react-router";
import { routeTree } from "./routeTree.gen";
import {
  RouteErrorBoundary,
  RoutePending,
  RouteNotFound,
} from "@/routes/-shared/route-states";

export const router = createRouter({
  routeTree,
  defaultPreload: "intent",
  defaultPreloadStaleTime: 0,
  defaultPendingComponent: RoutePending,
  defaultErrorComponent: RouteErrorBoundary,
  defaultNotFoundComponent: RouteNotFound,
  // Context-driven dependencies (TanStack Query client, auth, etc.)
  // are populated in __root.tsx via beforeLoad. Phase 0 has none.
  context: undefined!,
});

// Type registration — required for full type safety on Link/useNavigate.
declare module "@tanstack/react-router" {
  interface Register {
    router: typeof router;
  }
}
```

### Configuration choices

| Option | Value | Rationale |
|---|---|---|
| `defaultPreload` | `"intent"` | Preload on hover/focus. Cheap on a desktop app and improves perceived responsiveness between sections. |
| `defaultPreloadStaleTime` | `0` | Don't cache route preloads — TanStack Query caches data, the router only caches the route module. |
| `defaultPendingComponent` | `RoutePending` | Single skeleton component used across all routes |
| `defaultErrorComponent` | `RouteErrorBoundary` | Surface error to user with "Reset" action |
| `defaultNotFoundComponent` | `RouteNotFound` | Bookmark/typo fallback |

Per-route components can override any of these.

---

## 2. Root Route

The root route renders the platform shell (sidebar + header) defined in `ui-design-system.md` and provides the application's outermost error boundary.

### File: `apps/desktop/src/routes/__root.tsx`

```typescript
import { createRootRoute, Outlet, ScrollRestoration } from "@tanstack/react-router";
import { Suspense } from "react";
import { SidebarProvider } from "@chrdfin/ui/components/sidebar";
import { AppSidebar } from "@/components/shell/sidebar";
import { AppHeader } from "@/components/shell/header";
import { CommandPalette } from "@/components/shell/command-palette";

export const Route = createRootRoute({
  component: RootLayout,
});

function RootLayout(): JSX.Element {
  return (
    <SidebarProvider defaultOpen>
      <div className="flex h-screen w-screen overflow-hidden bg-background text-foreground">
        <AppSidebar />
        <div className="flex flex-1 flex-col overflow-hidden">
          <AppHeader />
          <main className="flex-1 overflow-auto">
            <Outlet />
          </main>
        </div>
        <CommandPalette />
        <ScrollRestoration />
      </div>
      <Suspense fallback={null}>
        <DevtoolsLazy />
      </Suspense>
    </SidebarProvider>
  );
}

/* ---------- Devtools (development only) ---------- */

const DevtoolsLazy =
  import.meta.env.PROD
    ? () => null
    : (await import("@tanstack/router-devtools")).TanStackRouterDevtools;
```

**Tradeoffs:**
- Root route uses `createRootRoute` (not `createFileRoute("/")`) — TanStack Router's special root constructor that doesn't have a path of its own.
- `<ScrollRestoration />` restores scroll position when navigating back to a previously-visited route. Useful when scrolling through a 200-row screener, switching to a ticker detail, then hitting back.
- Devtools loaded with a `top-level await import` guarded by `import.meta.env.PROD`. Vite tree-shakes the entire devtools package out of production builds. The cost in dev is one extra chunk that gets cached after first load.
- `defaultOpen` on `<SidebarProvider>` is the initial state for the very first launch. Subsequent sessions read the persisted state via the `ThemeProvider`-style Tauri command pattern (Phase 0 task — extend `<SidebarProvider>` with persistent state similar to `<ThemeProvider>`).

---

## 3. Section Layout Routes

Each of the four top-level sections (`analysis`, `tracking`, `tools`, `market`) has a `route.tsx` defining its layout and section-level error boundary.

### File: `apps/desktop/src/routes/analysis/route.tsx`

```typescript
import { createFileRoute, Outlet, redirect } from "@tanstack/react-router";
import { featureFlags, type FeatureFlag } from "@chrdfin/config";
import { SectionErrorBoundary } from "@/routes/-shared/route-states";

/**
 * Section-level required flags. If ALL of these are off, the section
 * itself is unreachable and the user is redirected to the home route.
 */
const SECTION_FLAGS: ReadonlyArray<FeatureFlag> = [
  "backtest",
  "monteCarlo",
  "optimizer",
];

export const Route = createFileRoute("/analysis")({
  beforeLoad: () => {
    const anyEnabled = SECTION_FLAGS.some((flag) => featureFlags[flag]);
    if (!anyEnabled) {
      throw redirect({ to: "/" });
    }
  },
  component: AnalysisSectionLayout,
  errorComponent: SectionErrorBoundary,
});

function AnalysisSectionLayout(): JSX.Element {
  return <Outlet />;
}
```

### File: `apps/desktop/src/routes/analysis/index.tsx`

```typescript
import { createFileRoute, redirect } from "@tanstack/react-router";
import { featureFlags } from "@chrdfin/config";

/**
 * /analysis itself has no content — redirect to the first enabled
 * sub-route. Order reflects priority: backtest > monte-carlo > optimizer.
 */
export const Route = createFileRoute("/analysis/")({
  beforeLoad: () => {
    if (featureFlags.backtest) {
      throw redirect({ to: "/analysis/backtest" });
    }
    if (featureFlags.monteCarlo) {
      throw redirect({ to: "/analysis/monte-carlo" });
    }
    if (featureFlags.optimizer) {
      throw redirect({ to: "/analysis/optimizer" });
    }
    throw redirect({ to: "/" });
  },
});
```

The other section layout files (`tracking/route.tsx`, `tools/route.tsx`, `market/route.tsx`) follow the same pattern with their respective flags.

**Tradeoffs:**
- Section-level error boundaries are intentional — a failure in `<EquityCurveWithDrawdown>` shouldn't bring down the portfolio tracker. The boundary scope matches the section taxonomy users already understand from the sidebar.
- Section-level redirects in `route.tsx` use `beforeLoad` rather than `loader` because they should fire before any data fetching. `beforeLoad` redirects propagate to the navigation engine without invoking the component.
- The two-tier check (section `route.tsx` + section `index.tsx`) is verbose but the alternative (single check on `index.tsx` only) leaves direct navigation to `/analysis/backtest` un-gated when the entire section should be off. Defense in depth.

---

## 4. Search Param Schemas

All search param schemas live in `@chrdfin/types`. Routes import and apply them via `validateSearch`. This centralizes the schemas so they can be referenced by:
- Route components (consume params)
- Navigation helpers (construct typed URLs)
- Tests (validate roundtrip serialization)
- Future export/share UI (deep-link generation)

### File: `packages/@chrdfin/types/src/search-params.ts`

```typescript
import { z } from "zod";

/* ============================================================
   Common atoms
   ============================================================ */

const isoDate = z
  .string()
  .regex(/^\d{4}-\d{2}-\d{2}$/, "Must be YYYY-MM-DD");

const tickerSymbol = z
  .string()
  .min(1)
  .max(10)
  .regex(/^[A-Z0-9.\-]+$/, "Invalid ticker format");

/**
 * Comma-separated list of tickers, max 50. Encoded as "AAPL,MSFT,GOOGL".
 * Decoded to ["AAPL", "MSFT", "GOOGL"].
 *
 * URL-safe by avoiding JSON encoding for arrays. Keeps shareable links
 * short and human-readable.
 */
const tickerList = z
  .string()
  .optional()
  .transform((s) => (s ? s.split(",").filter(Boolean) : []))
  .pipe(z.array(tickerSymbol).max(50));

/* ============================================================
   Backtest search params (/analysis/backtest)
   ============================================================ */

export const BacktestSearchSchema = z.object({
  /** Comma-separated tickers, e.g. "SPY,AGG" */
  tickers: tickerList.optional(),
  /** Comma-separated weights matching tickers, e.g. "0.6,0.4" */
  weights: z
    .string()
    .optional()
    .transform((s) => (s ? s.split(",").map(Number).filter((n) => !Number.isNaN(n)) : []))
    .pipe(z.array(z.number().min(0).max(1)).max(50)),
  start: isoDate.optional(),
  end: isoDate.optional(),
  rebalance: z
    .enum(["none", "monthly", "quarterly", "annually"])
    .optional()
    .default("annually"),
  /** Initial portfolio value */
  initial: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : undefined))
    .pipe(z.number().positive().optional()),
});

export type BacktestSearch = z.infer<typeof BacktestSearchSchema>;

/* ============================================================
   Monte Carlo search params (/analysis/monte-carlo)
   ============================================================ */

export const MonteCarloSearchSchema = z.object({
  initial: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 100_000))
    .pipe(z.number().positive()),
  paths: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 10_000))
    .pipe(z.number().int().min(100).max(100_000)),
  /** Horizon in years */
  horizon: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 30))
    .pipe(z.number().int().min(1).max(60)),
  /** Annualized expected return in percent */
  mu: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 7))
    .pipe(z.number().min(-50).max(50)),
  /** Annualized volatility in percent */
  sigma: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 15))
    .pipe(z.number().min(0).max(100)),
});

export type MonteCarloSearch = z.infer<typeof MonteCarloSearchSchema>;

/* ============================================================
   Portfolio tracker search params (/tracking/portfolio)
   ============================================================ */

export const PortfolioSearchSchema = z.object({
  /** Selected portfolio ID. If omitted, defaults to most-recent. */
  id: z.string().uuid().optional(),
});

export type PortfolioSearch = z.infer<typeof PortfolioSearchSchema>;

/* ============================================================
   Screener search params (/market/screener)
   ============================================================ */

export const ScreenerSearchSchema = z.object({
  asset: z.enum(["stocks", "etfs", "all"]).optional().default("stocks"),
  sectors: tickerList.optional(), // reuses the comma-separated string atom
  marketCapMin: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : undefined))
    .pipe(z.number().nonnegative().optional()),
  marketCapMax: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : undefined))
    .pipe(z.number().positive().optional()),
  yieldMin: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : undefined))
    .pipe(z.number().nonnegative().max(100).optional()),
  peMin: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : undefined))
    .pipe(z.number().optional()),
  peMax: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : undefined))
    .pipe(z.number().optional()),
  /** Sort column. Empty = default ranking. */
  sort: z
    .enum([
      "ticker", "marketCap", "price", "dayChange", "ytd",
      "yield", "pe", "volume",
    ])
    .optional(),
  /** Sort direction */
  dir: z.enum(["asc", "desc"]).optional().default("desc"),
});

export type ScreenerSearch = z.infer<typeof ScreenerSearchSchema>;

/* ============================================================
   News search params (/market/news)
   ============================================================ */

export const NewsSearchSchema = z.object({
  category: z.enum(["all", "earnings", "macro", "policy", "company"]).optional().default("all"),
  source: z.string().optional(),
  /** Filter by ticker — comma-separated */
  tickers: tickerList.optional(),
  /** Search query */
  q: z.string().max(200).optional(),
});

export type NewsSearch = z.infer<typeof NewsSearchSchema>;

/* ============================================================
   Calendar search params (/market/calendar)
   ============================================================ */

export const CalendarSearchSchema = z.object({
  view: z.enum(["earnings", "economic", "ipo", "splits"]).optional().default("earnings"),
  from: isoDate.optional(),
  to: isoDate.optional(),
});

export type CalendarSearch = z.infer<typeof CalendarSearchSchema>;
```

**Design choices:**
- **Comma-separated arrays** instead of repeated query params (`?tickers=AAPL&tickers=MSFT`). Shorter URLs, easier to read, easier to parse — at the cost of needing custom transforms.
- **`.optional().transform(...).pipe(...)`** pattern for numeric inputs. The chain is: accept missing → convert string → validate range. This handles URL semantics (everything is a string) while presenting typed numbers to consumers.
- **`.default(...)` on enums** instead of leaving them optional. Default values should appear in the typed object so route components don't have to repeat fallback logic everywhere.
- **No deeply nested objects.** Search params are flattened to maximize URL readability and minimize encoding overhead. If a future feature needs a complex config (e.g. multi-leg options strategy), serialize as a single base64-encoded string under one param rather than nesting.
- **Length limits on free-text fields** (`q.max(200)`) prevent malicious URL bloat — Tauri's webview is permissive but DuckDB queries derived from search params should be bounded.

### Helper: `zodValidator`

A small adapter for using Zod schemas with TanStack Router's `validateSearch`. TanStack accepts any `(input: Record<string, unknown>) => T` function.

### File: `packages/@chrdfin/types/src/lib/zod-validator.ts`

```typescript
import type { z } from "zod";

/**
 * Adapt a Zod schema to TanStack Router's validateSearch signature.
 *
 * Throws on validation failure, which TanStack Router's error boundary
 * catches and surfaces. Invalid search params on a bookmarked URL will
 * land the user on the section's error component with a descriptive
 * message.
 *
 * @example
 *   export const Route = createFileRoute("/analysis/backtest")({
 *     validateSearch: zodValidator(BacktestSearchSchema),
 *     ...
 *   });
 */
export function zodValidator<T extends z.ZodTypeAny>(schema: T) {
  return (input: Record<string, unknown>): z.infer<T> => schema.parse(input);
}
```

---

## 5. Leaf Routes

### Pattern A: Static placeholder (Phase 0)

Most Phase 0 leaf routes render a "Coming in Phase N" placeholder. The actual implementation lives in the lazy chunk.

#### File: `apps/desktop/src/routes/analysis/backtest.tsx`

```typescript
import { createFileRoute } from "@tanstack/react-router";
import { zodValidator, BacktestSearchSchema } from "@chrdfin/types";

export const Route = createFileRoute("/analysis/backtest")({
  validateSearch: zodValidator(BacktestSearchSchema),
});
```

#### File: `apps/desktop/src/routes/analysis/backtest.lazy.tsx`

```typescript
import { createLazyFileRoute } from "@tanstack/react-router";
import { PhasePlaceholder } from "@/routes/-shared/route-states";

export const Route = createLazyFileRoute("/analysis/backtest")({
  component: BacktestPage,
});

function BacktestPage(): JSX.Element {
  // Phase 0: placeholder. Phase 2-3 swaps in real implementation.
  return <PhasePlaceholder phase={2} feature="Backtesting" />;
}
```

The `*.tsx` file declares the route metadata (path, search validator, loaders). The `*.lazy.tsx` file declares only the component. TanStack Router's plugin merges these at compile time — the metadata is in the eagerly-loaded route tree, the component is in a deferred chunk.

When Phase 2 ships, only `backtest.lazy.tsx` changes; `backtest.tsx` stays the same. This is the boundary that lets feature work proceed without touching route plumbing.

### Pattern B: Search-param-driven page (Phase 2+)

The Phase 2 implementation reads validated search params and threads them into the page logic.

#### File: `apps/desktop/src/routes/analysis/backtest.lazy.tsx` (Phase 2 version)

```typescript
import { createLazyFileRoute } from "@tanstack/react-router";
import { useMemo } from "react";
import { useTauriQuery } from "@/hooks/use-tauri-command";
import { EquityCurveWithDrawdown } from "@chrdfin/charts";
import { MetricsStrip, Metric } from "@chrdfin/ui";
import type { BacktestResult } from "@chrdfin/types";

export const Route = createLazyFileRoute("/analysis/backtest")({
  component: BacktestPage,
});

function BacktestPage(): JSX.Element {
  const { tickers, weights, start, end, rebalance, initial } = Route.useSearch();

  // Build the config object the Rust backend expects.
  const config = useMemo(
    () =>
      tickers && tickers.length > 0
        ? { tickers, weights, start, end, rebalance, initial: initial ?? 10_000 }
        : null,
    [tickers, weights, start, end, rebalance, initial],
  );

  const { data, isLoading, error } = useTauriQuery<BacktestResult>(
    "run_backtest",
    config ?? {},
    { enabled: config !== null, staleTime: Infinity },
  );

  if (config === null) {
    return <BacktestEmptyForm />;
  }
  if (isLoading) {
    return <div className="p-6 text-xs text-muted-foreground">Running backtest...</div>;
  }
  if (error) {
    return (
      <div className="p-6 text-xs">
        <span className="text-destructive">Backtest failed: </span>
        <span className="text-muted-foreground">{error.message}</span>
      </div>
    );
  }
  if (!data) return null;

  return (
    <div className="flex flex-col">
      <MetricsStrip>
        <Metric label="CAGR" delta={{ value: data.cagr, format: "percent" }} />
        <Metric label="Total Return" delta={{ value: data.totalReturn, format: "percent" }} />
        {/* ... other metrics ... */}
      </MetricsStrip>
      <EquityCurveWithDrawdown data={data.timeline} title="Equity Curve" />
    </div>
  );
}

function BacktestEmptyForm(): JSX.Element {
  return (
    <div className="p-6 text-xs text-muted-foreground">
      Enter a portfolio configuration to begin.
    </div>
  );
}
```

`Route.useSearch()` is the typed accessor from the route's own definition. Because `validateSearch: zodValidator(BacktestSearchSchema)` is declared in the metadata file, the inferred type carries through to this lazy component.

### Pattern C: Dynamic path parameter

#### File: `apps/desktop/src/routes/market/ticker/$symbol.tsx`

```typescript
import { createFileRoute, redirect } from "@tanstack/react-router";

export const Route = createFileRoute("/market/ticker/$symbol")({
  // Path params are validated via parseParams.
  parseParams: ({ symbol }) => {
    const upper = symbol.toUpperCase();
    if (!/^[A-Z0-9.\-]{1,10}$/.test(upper)) {
      throw redirect({ to: "/market/screener" });
    }
    return { symbol: upper };
  },
  // Stringify enforces uppercase in the URL when navigating programmatically.
  stringifyParams: ({ symbol }) => ({ symbol: symbol.toUpperCase() }),
});
```

#### File: `apps/desktop/src/routes/market/ticker/$symbol.lazy.tsx`

```typescript
import { createLazyFileRoute } from "@tanstack/react-router";

export const Route = createLazyFileRoute("/market/ticker/$symbol")({
  component: TickerDetailPage,
});

function TickerDetailPage(): JSX.Element {
  const { symbol } = Route.useParams();
  // ... fetch and render ticker detail
  return <div className="p-6">{symbol}</div>;
}
```

**Tradeoffs:**
- Path param validation in `parseParams` rather than `beforeLoad`. `parseParams` runs synchronously and returns a typed object that's passed to all child routes; `beforeLoad` is for async work.
- Symbols are normalized to uppercase. Users typing `aapl` in the address bar get redirected to `AAPL` via `stringifyParams`. This means the URL is canonical regardless of how the user navigated.
- Invalid symbols redirect to the screener (graceful) rather than throwing (which would land the user on the section error boundary). `parseParams` calling `redirect()` is supported in TanStack Router 1.x+.

---

## 6. Pending & Error Components

The platform-wide pending and error components matching the Bloomberg-terminal aesthetic. Used as `defaultPendingComponent` / `defaultErrorComponent` in the router config and overrideable per-route.

### File: `apps/desktop/src/routes/-shared/route-states.tsx`

```typescript
import type { ErrorComponentProps } from "@tanstack/react-router";
import { Link } from "@tanstack/react-router";

/* ============================================================
   Pending
   ============================================================ */

/**
 * Default pending state. Empty space with subtle text — no spinner.
 *
 * Per the design system, charts and dense UI display loading text
 * inline rather than blocking the entire route with a skeleton. Routes
 * with significant load delay (Tauri-side computation) override this.
 */
export function RoutePending(): JSX.Element {
  return (
    <div className="flex h-full items-center justify-center text-xs text-muted-foreground">
      Loading...
    </div>
  );
}

/* ============================================================
   Error
   ============================================================ */

/**
 * Root-level error boundary. Catches anything not handled by a
 * section-level boundary. Provides a "Reset" action that clears
 * route state via the router instance.
 */
export function RouteErrorBoundary({
  error,
  reset,
}: ErrorComponentProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-md font-medium text-destructive">Something went wrong</div>
      <pre className="max-w-2xl text-xs text-muted-foreground whitespace-pre-wrap font-mono">
        {error instanceof Error ? error.message : String(error)}
      </pre>
      <div className="flex gap-2 pt-2">
        <button
          type="button"
          onClick={reset}
          className="border border-border px-3 py-1 text-xs hover:bg-accent"
        >
          Reset
        </button>
        <Link
          to="/"
          className="border border-border px-3 py-1 text-xs hover:bg-accent"
        >
          Home
        </Link>
      </div>
    </div>
  );
}

/**
 * Section-level error boundary. Same UI as root but the "Home" link
 * goes to the parent section index rather than the global root —
 * lets the user retry within the current section.
 */
export function SectionErrorBoundary({
  error,
  reset,
}: ErrorComponentProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-md font-medium text-destructive">Section unavailable</div>
      <pre className="max-w-2xl text-xs text-muted-foreground whitespace-pre-wrap font-mono">
        {error instanceof Error ? error.message : String(error)}
      </pre>
      <button
        type="button"
        onClick={reset}
        className="border border-border px-3 py-1 text-xs hover:bg-accent"
      >
        Retry
      </button>
    </div>
  );
}

/* ============================================================
   Not found
   ============================================================ */

export function RouteNotFound(): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-3 p-6">
      <div className="text-md font-medium">Route not found</div>
      <Link
        to="/"
        className="border border-border px-3 py-1 text-xs hover:bg-accent"
      >
        Home
      </Link>
    </div>
  );
}

/* ============================================================
   Phase placeholder
   ============================================================ */

export interface PhasePlaceholderProps {
  phase: number;
  feature: string;
}

/**
 * Phase 0 placeholder for routes whose feature ships in a later phase.
 * Renders a centered message with the phase number — matches the
 * "feature flag off" treatment used by the sidebar gate.
 */
export function PhasePlaceholder({
  phase,
  feature,
}: PhasePlaceholderProps): JSX.Element {
  return (
    <div className="flex h-full flex-col items-center justify-center gap-2 text-center">
      <div className="text-md font-medium">{feature}</div>
      <div className="text-xs text-muted-foreground">
        Coming in Phase {phase}.
      </div>
    </div>
  );
}
```

**Tradeoffs:**
- No spinners anywhere — text-only loading states match the platform aesthetic. The exception would be Tauri commands that take >5 seconds (Monte Carlo with 100k paths); those routes can override `pendingComponent` with a determinate progress bar fed by Tauri events (`useProgress` hook, defined per `apps/desktop/src/hooks/use-progress.ts` in CLAUDE.md).
- Error message exposes `error.message` directly. The Rust backend is responsible for translating internal errors to user-readable strings via `thiserror` per CLAUDE.md. If a low-level Rust error leaks unprocessed (e.g. `RusqliteError(...)`) that's a backend bug, not a UI bug.
- "Reset" calls TanStack Router's `reset` callback, which forces re-evaluation of the failing route. This handles transient failures (Tauri command failed once, succeeds on retry).

---

## 7. Type-Safe Navigation

TanStack Router exports a typed `<Link>` and `useNavigate()`. With the router instance registered (per section 1), every `to` prop is autocompleted and `params` / `search` are checked against the target route's schema.

### Direct usage

```typescript
import { Link } from "@tanstack/react-router";

function Example(): JSX.Element {
  return (
    <>
      {/* Static path */}
      <Link to="/analysis/backtest">Backtest</Link>

      {/* Dynamic params */}
      <Link
        to="/market/ticker/$symbol"
        params={{ symbol: "AAPL" }}
      >
        AAPL
      </Link>

      {/* Search params */}
      <Link
        to="/analysis/backtest"
        search={{
          tickers: ["SPY", "AGG"],
          weights: [0.6, 0.4],
          rebalance: "annually",
        }}
      >
        60/40 Backtest
      </Link>
    </>
  );
}
```

### Programmatic navigation

```typescript
import { useNavigate } from "@tanstack/react-router";

function ScreenerRow({ symbol }: { symbol: string }): JSX.Element {
  const navigate = useNavigate();
  return (
    <button
      onClick={() => navigate({
        to: "/market/ticker/$symbol",
        params: { symbol },
      })}
    >
      View {symbol}
    </button>
  );
}
```

### Domain-specific navigation hooks

For sections with frequent programmatic navigation (screener row clicks → ticker detail; backtest config form → URL update), wrap `useNavigate()` in a domain hook to avoid repeating `to`/`from` strings throughout the codebase.

#### File: `apps/desktop/src/hooks/use-domain-nav.ts`

```typescript
import { useNavigate, useSearch } from "@tanstack/react-router";
import { useCallback } from "react";
import type { BacktestSearch, ScreenerSearch } from "@chrdfin/types";

/* ---------- Backtest ---------- */

export function useBacktestNav(): {
  search: BacktestSearch;
  updateSearch: (next: Partial<BacktestSearch>) => void;
} {
  const search = useSearch({ from: "/analysis/backtest" });
  const navigate = useNavigate({ from: "/analysis/backtest" });

  const updateSearch = useCallback(
    (next: Partial<BacktestSearch>) => {
      navigate({ search: (prev) => ({ ...prev, ...next }) });
    },
    [navigate],
  );

  return { search, updateSearch };
}

/* ---------- Screener ---------- */

export function useScreenerNav(): {
  search: ScreenerSearch;
  updateSearch: (next: Partial<ScreenerSearch>) => void;
  resetFilters: () => void;
} {
  const search = useSearch({ from: "/market/screener" });
  const navigate = useNavigate({ from: "/market/screener" });

  const updateSearch = useCallback(
    (next: Partial<ScreenerSearch>) => {
      navigate({ search: (prev) => ({ ...prev, ...next }) });
    },
    [navigate],
  );

  const resetFilters = useCallback(() => {
    navigate({ search: () => ({}) });
  }, [navigate]);

  return { search, updateSearch, resetFilters };
}
```

**Tradeoffs:**
- The hooks aren't strictly necessary — `useSearch` and `useNavigate` already provide the same functionality. They exist because:
  1. Page components consume search params in 5-10 places; centralizing the `from` string avoids drift.
  2. `updateSearch` with merge-style updates is the dominant pattern; the hook bakes in the merge so consumers don't repeat it.
  3. Domain-specific helpers (`resetFilters`, `incrementPage`) accumulate naturally here.
- One hook per route, not per section. A section-level hook would obscure which route's search schema is in scope.

---

## 8. Loaders

For routes that need data preloaded before component render, use TanStack Router's `loader` API. Most chrdfin routes do **not** use loaders — TanStack Query handles data fetching inside components, which gives finer-grained loading states.

Loaders are appropriate when:
- The route is unusable without specific data (e.g. ticker detail page must have ticker metadata)
- The data should be preloaded on hover (`defaultPreload: "intent"` will trigger the loader)
- You want to fail-fast at the route boundary rather than render a partial UI

### Example: ticker detail loader

```typescript
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import type { TickerMetadata } from "@chrdfin/types";

export const Route = createFileRoute("/market/ticker/$symbol")({
  parseParams: ({ symbol }) => ({ symbol: symbol.toUpperCase() }),
  loader: async ({ params }): Promise<{ metadata: TickerMetadata }> => {
    // This data MUST exist for the route to function. Failure throws
    // and lands at the section error boundary.
    const metadata = await invoke<TickerMetadata>("get_ticker_metadata", {
      symbol: params.symbol,
    });
    return { metadata };
  },
  staleTime: 5 * 60 * 1000, // 5 minutes
});
```

The loaded data is accessed via `Route.useLoaderData()` in the component:

```typescript
function TickerDetailPage(): JSX.Element {
  const { metadata } = Route.useLoaderData();
  // ... render
}
```

**Tradeoffs:**
- `staleTime: 5 * 60 * 1000` means the loader is only re-invoked if the user navigates away and back after 5 minutes. Within that window, `defaultPreload: "intent"` will use the cached result.
- Loaders bypass TanStack Query's cache. If the same data is also fetched via `useTauriQuery` elsewhere, you have two caches. For ticker metadata that's mostly route-local, this is fine. For shared data (price quotes referenced from multiple routes), prefer `useTauriQuery` and skip the loader.

---

## 9. Code Splitting Strategy

The route tree's split boundaries determine bundle granularity:

| Boundary | Loaded eagerly | Loaded lazily |
|---|---|---|
| Initial load | `__root.tsx`, `index.tsx`, `routeTree.gen.ts`, all `*.tsx` route metadata files | Nothing else |
| First navigation to `/analysis/*` | `analysis/route.tsx` metadata (already loaded) | `analysis/<leaf>.lazy.tsx` for the visited leaf |
| Hover on a sidebar nav link | (preloaded by `defaultPreload: "intent"`) | The corresponding leaf chunk |

Because every route metadata file is in the eager bundle, the initial load is bounded by:
- The route tree itself (~20KB)
- The `__root.tsx` shell (sidebar, header, command palette)
- The `index.tsx` home view
- All Zod schemas referenced by `validateSearch` calls

Each lazy leaf is a separate chunk, typically 15-50KB depending on dependencies (charts, complex components). A user who only ever opens the portfolio tracker never loads backtest, Monte Carlo, screener, etc.

### Verifying chunk boundaries

After `pnpm tauri build`, inspect the Vite bundle output:

```bash
pnpm --filter desktop build
ls apps/desktop/dist/assets/
# Should show:
#   index-[hash].js              (eager bundle)
#   backtest-[hash].js           (lazy chunk)
#   monte-carlo-[hash].js        (lazy chunk)
#   ticker-[hash].js             (lazy chunk)
#   ... etc.
```

Each lazy `*.lazy.tsx` file should produce one chunk named after its route. If chunks don't appear, the lazy declaration is misconfigured — usually a missing `createLazyFileRoute` import or a stray `import { Component } from './leaf.lazy'` in an eager file.

### Bundle budget

Phase 0 target: initial bundle ≤ 250KB gzipped (excluding fonts).

Per-leaf budget (loose):
- Simple list pages (transactions, news): ≤ 30KB
- Chart-heavy pages (backtest, monte-carlo): ≤ 80KB (Recharts is ~50KB)
- Dense interactive pages (screener, portfolio): ≤ 100KB

If a single chunk exceeds 150KB, it's a candidate for further splitting (e.g. extract chart-only sub-components to nested lazy components).

---

## 10. Feature Flag Integration

Feature flags from `@chrdfin/config` gate route accessibility at three points:

| Point | Mechanism | Effect |
|---|---|---|
| Sidebar | `featureFlags[flag]` filter | Nav item not rendered (per `ui-component-recipes.md` section 12) |
| Section route | `beforeLoad` redirect | Direct navigation to section root redirects home |
| Leaf route (optional) | `beforeLoad` check | Direct navigation to disabled leaf shows fallback |

For Phase 0 the section-level gate is sufficient — direct navigation to a disabled leaf renders the lazy chunk's `<PhasePlaceholder>`, which is the right UX (the user sees "Coming in Phase X" instead of an error).

For later phases where some leaves go from "shipped" to "disabled" (e.g. running with `optimizer: false` after Phase 9 ships), add per-leaf gates:

### Optional: `<FeatureGate>` wrapper

#### File: `apps/desktop/src/routes/-shared/feature-gate.tsx`

```typescript
import type { ReactNode } from "react";
import { featureFlags, type FeatureFlag } from "@chrdfin/config";
import { PhasePlaceholder } from "./route-states";

export interface FeatureGateProps {
  flag: FeatureFlag;
  /** Phase number to show in fallback if feature is disabled. */
  phase: number;
  /** Feature display name for the fallback. */
  feature: string;
  children: ReactNode;
}

/**
 * Wrap a route component to gate its rendering on a feature flag.
 *
 * @example
 *   function OptimizerPage(): JSX.Element {
 *     return (
 *       <FeatureGate flag="optimizer" phase={9} feature="Optimizer">
 *         <OptimizerImpl />
 *       </FeatureGate>
 *     );
 *   }
 */
export function FeatureGate({
  flag,
  phase,
  feature,
  children,
}: FeatureGateProps): JSX.Element {
  if (!featureFlags[flag]) {
    return <PhasePlaceholder phase={phase} feature={feature} />;
  }
  return <>{children}</>;
}
```

---

## 11. Common Pitfalls

| Symptom | Cause | Fix |
|---|---|---|
| `routeTree.gen.ts` not regenerating | Vite plugin not running (e.g. `pnpm typecheck` standalone) | Run `pnpm dev` or `pnpm build` first; CI must run plugin before typecheck |
| Type errors on `Link to` | Router not registered via `declare module` | Verify `router.tsx` has the module augmentation block |
| `useSearch()` returns `unknown` | Missing `from` parameter, or using outside a route's component | Pass `from: "/path"` explicitly, or use `Route.useSearch()` from inside the route's lazy file |
| Lazy chunk loaded eagerly | `*.tsx` metadata file imports from `*.lazy.tsx` | Metadata files can only declare metadata; component code lives only in `*.lazy.tsx` |
| Search params lost on navigation | Missing `search: (prev) => ({...prev, ...})` merge | Always merge previous search when partially updating |
| Path param case mismatch | URL has `/aapl` but params expect uppercase | Normalize in `parseParams` and provide `stringifyParams` |
| Section error boundary not firing | Error thrown in lazy component, not caught by section | Section `errorComponent` only catches synchronous errors; for async errors use TanStack Query's error states inside the component |
| Feature flag changes don't reflect after toggle | Vite HMR doesn't reload `@chrdfin/config` | Hard refresh (Cmd/Ctrl+R inside webview) or restart `pnpm tauri dev` |
| Devtools shipping in production | `import.meta.env.PROD` not checked | Wrap devtools import in env guard; verify build output excludes the chunk |
| Redirects in `beforeLoad` infinite-loop | Target route's `beforeLoad` redirects back | Prefer leaf paths in section index redirects; never redirect from a section's `route.tsx` to one of its children if that child might redirect back |
| `validateSearch` throwing on bookmarked URL | Schema requires fields the URL omits | Make all schema fields optional with `.default()` where reasonable; mark truly required fields with explicit error message |

---

## 12. Testing Patterns

Routing logic is covered at three levels.

### Level 1: Search param schema tests

Pure schema tests live next to the schema definitions in `@chrdfin/types`.

```typescript
// packages/@chrdfin/types/src/search-params.test.ts
import { describe, it, expect } from "vitest";
import { BacktestSearchSchema, ScreenerSearchSchema } from "./search-params";

describe("BacktestSearchSchema", () => {
  it("parses comma-separated tickers", () => {
    const result = BacktestSearchSchema.parse({ tickers: "SPY,AGG" });
    expect(result.tickers).toEqual(["SPY", "AGG"]);
  });

  it("parses comma-separated weights to numbers", () => {
    const result = BacktestSearchSchema.parse({ weights: "0.6,0.4" });
    expect(result.weights).toEqual([0.6, 0.4]);
  });

  it("applies default rebalance frequency", () => {
    const result = BacktestSearchSchema.parse({});
    expect(result.rebalance).toBe("annually");
  });

  it("rejects invalid ticker formats", () => {
    expect(() => BacktestSearchSchema.parse({ tickers: "spy,agg" })).toThrow();
  });

  it("validates date format", () => {
    expect(() => BacktestSearchSchema.parse({ start: "2020/01/01" })).toThrow();
  });
});

describe("ScreenerSearchSchema", () => {
  it("clamps yieldMin to non-negative", () => {
    expect(() => ScreenerSearchSchema.parse({ yieldMin: "-1" })).toThrow();
  });

  it("preserves sort direction default", () => {
    expect(ScreenerSearchSchema.parse({}).dir).toBe("desc");
  });
});
```

### Level 2: Component tests with router fixtures

Use TanStack Router's testing utilities to render routes in isolation.

```typescript
// apps/desktop/src/routes/__tests__/backtest-search.test.tsx
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import {
  createMemoryHistory,
  createRootRoute,
  createRoute,
  createRouter,
  RouterProvider,
} from "@tanstack/react-router";
import { zodValidator, BacktestSearchSchema } from "@chrdfin/types";

function makeTestRouter(initialPath: string) {
  const rootRoute = createRootRoute();
  const backtestRoute = createRoute({
    getParentRoute: () => rootRoute,
    path: "/analysis/backtest",
    validateSearch: zodValidator(BacktestSearchSchema),
    component: function TestComponent() {
      const search = backtestRoute.useSearch();
      return <pre data-testid="search">{JSON.stringify(search)}</pre>;
    },
  });

  return createRouter({
    routeTree: rootRoute.addChildren([backtestRoute]),
    history: createMemoryHistory({ initialEntries: [initialPath] }),
  });
}

describe("/analysis/backtest search params", () => {
  it("parses tickers and weights from URL", async () => {
    const router = makeTestRouter(
      "/analysis/backtest?tickers=SPY,AGG&weights=0.6,0.4",
    );
    const { findByTestId } = render(<RouterProvider router={router} />);
    const el = await findByTestId("search");
    const parsed = JSON.parse(el.textContent ?? "{}");
    expect(parsed.tickers).toEqual(["SPY", "AGG"]);
    expect(parsed.weights).toEqual([0.6, 0.4]);
  });
});
```

### Level 3: End-to-end is out of scope

For a single-user desktop app, full E2E (Playwright, Tauri test runners) is deferred. Vitest covers schema and routing logic; manual smoke testing covers the integrated flow.

---

## 13. References

- TanStack Router docs: https://tanstack.com/router/latest/docs/framework/react/overview
- File-based routing guide: https://tanstack.com/router/latest/docs/framework/react/guide/file-based-routing
- Search params guide: https://tanstack.com/router/latest/docs/framework/react/guide/search-params
- Code splitting guide: https://tanstack.com/router/latest/docs/framework/react/guide/code-splitting
- Vite plugin: https://tanstack.com/router/latest/docs/framework/react/guide/router-plugin
- Zod docs: https://zod.dev

---

## 14. Document Maintenance

When adding a new route:
1. Determine the section. Add the file under the appropriate section directory.
2. If the route accepts search params, define the schema in `@chrdfin/types/src/search-params.ts` first.
3. Create the `*.tsx` metadata file with `validateSearch` (if applicable) and `parseParams` (for dynamic params).
4. Create the `*.lazy.tsx` companion with the component.
5. Add the route to the file layout table at the top of this document.
6. If the route requires a feature flag check, decide whether section-level gating is sufficient or if a per-leaf gate is needed.
7. Run `pnpm typecheck` to verify the route tree typing is sound.
8. Add at least one schema test if search params are involved.

When restructuring routes:
1. Avoid renaming paths post-Phase-0 — bookmarked URLs and shared deep-links break. If a rename is unavoidable, add a redirect from the old path in the section's `route.tsx`.
2. Section restructures (moving a domain between sections) require updating: the file location, the sidebar nav table, the section flag list, and any cross-route `<Link>` references.
3. Path parameter additions (e.g. `/market/ticker/$symbol/options` → `/market/ticker/$symbol/options/$expiry`) are non-breaking if old paths still resolve. Removing path segments is breaking.
