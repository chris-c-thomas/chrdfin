# UI Component Recipes — chrdfin

## Purpose

Concrete, production-ready implementations for the components, hooks, and utilities referenced in `docs/ui-design-system.md`. This file is the implementation contract: Claude Code agents writing Phase 0 (and later) UI code copy from these recipes verbatim, then extend.

Every recipe is:
- Production-ready (no pseudocode, no `// TODO` placeholders)
- Strictly typed (TypeScript strict mode, no `any`)
- Tested (Vitest examples included for non-trivial logic)
- Failure-mode aware (edge cases called out)
- Aligned with the architectural constraints in `CLAUDE.md` (no `localStorage`, no default exports, named imports, kebab-case module files, etc.)

**Companion documents:**
- `docs/ui-design-system.md` — design philosophy, color tokens, density rules
- `docs/technical-blueprint.md` — system architecture, package boundaries
- `docs/phase-0-checklist.md` — Phase 0 implementation tasks
- `docs/type-definitions-reference.md` — shared types in `@chrdfin/types`
- `docs/database-schema-reference.md` — DuckDB schema (referenced for `settings` table)

---

## Package Boundaries

Each recipe declares its target package and file path. Adhere strictly — cross-package imports must respect the dependency graph in CLAUDE.md.

| Recipe | Package | File Path |
|---|---|---|
| Number formatting utilities | `@chrdfin/ui` | `packages/@chrdfin/ui/src/lib/format.ts` |
| `cn()` utility | `@chrdfin/ui` | `packages/@chrdfin/ui/src/lib/utils.ts` |
| `useThemeColors` hook | `@chrdfin/charts` | `packages/@chrdfin/charts/src/hooks/use-theme-colors.ts` |
| `useMarketStatus` hook | `apps/desktop` | `apps/desktop/src/hooks/use-market-status.ts` |
| `useTauriCommand` hook | `apps/desktop` | `apps/desktop/src/hooks/use-tauri-command.ts` |
| `<DeltaValue>` | `@chrdfin/ui` | `packages/@chrdfin/ui/src/components/delta-value.tsx` |
| `<MetricsStrip>` | `@chrdfin/ui` | `packages/@chrdfin/ui/src/components/metrics-strip.tsx` |
| `<DataTable>` | `@chrdfin/ui` | `packages/@chrdfin/ui/src/components/data-table.tsx` |
| `<Sparkline>` | `@chrdfin/charts` | `packages/@chrdfin/charts/src/components/sparkline.tsx` |
| `<ThemeProvider>` | `apps/desktop` | `apps/desktop/src/components/providers/theme-provider.tsx` |
| `<MarketStatusIndicator>` | `apps/desktop` | `apps/desktop/src/components/shell/market-status-indicator.tsx` |

---

## 1. Foundation: Number Formatting

Every numeric display in the application routes through these functions. Centralizing formatting is what makes "+$2,841.07 / +0.34%" identical across the dashboard, screener, and backtest results without per-component drift.

### File: `packages/@chrdfin/ui/src/lib/format.ts`

```typescript
/**
 * Number formatting utilities for chrdfin.
 *
 * All functions are pure and deterministic. They use `Intl.NumberFormat`
 * with explicit locale (`en-US`) so output is identical across machines
 * and OS locale settings — important because chrdfin is desktop-only and
 * inconsistent locale behavior across platforms would corrupt screenshots,
 * test snapshots, and exported reports.
 *
 * All formatters accept `null` and `undefined` and return a placeholder
 * string (default: "—"). This is intentional — financial data has gaps
 * (illiquid securities, unreported metrics, pre-IPO history) and throwing
 * on null would cascade through every table cell.
 */

/* ---------- Types ---------- */

export interface FormatOptions {
  /** String to display when value is null/undefined/NaN. Default "—". */
  placeholder?: string;
  /** Show explicit "+" prefix on positive values. Default false. */
  signed?: boolean;
}

export interface CurrencyOptions extends FormatOptions {
  /** ISO 4217 currency code. Default "USD". */
  currency?: string;
  /** Decimal places. Default 2. */
  precision?: number;
  /** Wrap negatives in parentheses (accounting style). Default false. */
  accounting?: boolean;
}

export interface PercentOptions extends FormatOptions {
  /** Decimal places. Default 2. */
  precision?: number;
  /**
   * Whether the input is already in percent form (e.g. 34.7 = 34.7%)
   * or in decimal form (e.g. 0.347 = 34.7%). Default "percent".
   */
  scale?: "percent" | "decimal";
}

export interface NumberFormatOptions extends FormatOptions {
  /** Decimal places. Default 2. */
  precision?: number;
}

/* ---------- Internal helpers ---------- */

const DEFAULT_PLACEHOLDER = "—";

function isFiniteNumber(value: number | null | undefined): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function applySign(formatted: string, value: number, signed: boolean): string {
  if (!signed) return formatted;
  if (value > 0 && !formatted.startsWith("+") && !formatted.startsWith("-")) {
    return `+${formatted}`;
  }
  return formatted;
}

/* ---------- Currency ---------- */

const currencyFormatters = new Map<string, Intl.NumberFormat>();

function getCurrencyFormatter(
  currency: string,
  precision: number,
): Intl.NumberFormat {
  const key = `${currency}:${precision}`;
  let fmt = currencyFormatters.get(key);
  if (!fmt) {
    fmt = new Intl.NumberFormat("en-US", {
      style: "currency",
      currency,
      minimumFractionDigits: precision,
      maximumFractionDigits: precision,
    });
    currencyFormatters.set(key, fmt);
  }
  return fmt;
}

/**
 * Format a number as currency.
 *
 * @example
 *   formatCurrency(847293.14)              // "$847,293.14"
 *   formatCurrency(-1234.56)               // "-$1,234.56"
 *   formatCurrency(-1234.56, { accounting: true })  // "($1,234.56)"
 *   formatCurrency(2841.07, { signed: true })       // "+$2,841.07"
 *   formatCurrency(null)                   // "—"
 */
export function formatCurrency(
  value: number | null | undefined,
  options: CurrencyOptions = {},
): string {
  const {
    placeholder = DEFAULT_PLACEHOLDER,
    currency = "USD",
    precision = 2,
    accounting = false,
    signed = false,
  } = options;

  if (!isFiniteNumber(value)) return placeholder;

  const formatter = getCurrencyFormatter(currency, precision);

  if (accounting && value < 0) {
    return `(${formatter.format(Math.abs(value))})`;
  }

  return applySign(formatter.format(value), value, signed);
}

/* ---------- Percent ---------- */

const percentFormatters = new Map<number, Intl.NumberFormat>();

function getPercentFormatter(precision: number): Intl.NumberFormat {
  let fmt = percentFormatters.get(precision);
  if (!fmt) {
    fmt = new Intl.NumberFormat("en-US", {
      minimumFractionDigits: precision,
      maximumFractionDigits: precision,
    });
    percentFormatters.set(precision, fmt);
  }
  return fmt;
}

/**
 * Format a number as a percentage. The "%" suffix is appended directly.
 *
 * Note: defaults to `scale: "percent"` (input is already scaled to 100),
 * which is the convention used by the Rust computation engine. Use
 * `scale: "decimal"` if your input is a raw ratio (0.347).
 *
 * @example
 *   formatPercent(34.7)                          // "34.70%"
 *   formatPercent(34.7, { signed: true })        // "+34.70%"
 *   formatPercent(0.347, { scale: "decimal" })   // "34.70%"
 *   formatPercent(-22.1, { precision: 1 })       // "-22.1%"
 */
export function formatPercent(
  value: number | null | undefined,
  options: PercentOptions = {},
): string {
  const {
    placeholder = DEFAULT_PLACEHOLDER,
    precision = 2,
    scale = "percent",
    signed = false,
  } = options;

  if (!isFiniteNumber(value)) return placeholder;

  const scaled = scale === "decimal" ? value * 100 : value;
  const formatter = getPercentFormatter(precision);
  const formatted = `${formatter.format(scaled)}%`;

  return applySign(formatted, scaled, signed);
}

/* ---------- Plain numbers ---------- */

const numberFormatters = new Map<number, Intl.NumberFormat>();

function getNumberFormatter(precision: number): Intl.NumberFormat {
  let fmt = numberFormatters.get(precision);
  if (!fmt) {
    fmt = new Intl.NumberFormat("en-US", {
      minimumFractionDigits: precision,
      maximumFractionDigits: precision,
    });
    numberFormatters.set(precision, fmt);
  }
  return fmt;
}

/**
 * Format a plain number with thousands separators.
 *
 * @example
 *   formatNumber(12345.678)               // "12,345.68"
 *   formatNumber(1000, { precision: 0 })  // "1,000"
 */
export function formatNumber(
  value: number | null | undefined,
  options: NumberFormatOptions = {},
): string {
  const {
    placeholder = DEFAULT_PLACEHOLDER,
    precision = 2,
    signed = false,
  } = options;

  if (!isFiniteNumber(value)) return placeholder;

  const formatter = getNumberFormatter(precision);
  return applySign(formatter.format(value), value, signed);
}

/* ---------- Abbreviated (compact) ---------- */

interface AbbreviationStep {
  threshold: number;
  divisor: number;
  suffix: string;
}

const ABBREVIATIONS: ReadonlyArray<AbbreviationStep> = [
  { threshold: 1e12, divisor: 1e12, suffix: "T" },
  { threshold: 1e9, divisor: 1e9, suffix: "B" },
  { threshold: 1e6, divisor: 1e6, suffix: "M" },
  { threshold: 1e3, divisor: 1e3, suffix: "K" },
];

/**
 * Format a number in abbreviated form. Used for chart axes, market cap,
 * volume, and any context where horizontal space is constrained.
 *
 * Reserved for SECONDARY data display — never use abbreviation in primary
 * data columns where precision matters (P&L, share count, transaction
 * amounts, anything the user might mentally arithmetic against).
 *
 * @example
 *   formatAbbreviated(2_400_000_000_000)              // "2.40T"
 *   formatAbbreviated(12_400_000, { prefix: "$" })    // "$12.40M"
 *   formatAbbreviated(847.32)                         // "847.32"
 */
export function formatAbbreviated(
  value: number | null | undefined,
  options: NumberFormatOptions & { prefix?: string } = {},
): string {
  const {
    placeholder = DEFAULT_PLACEHOLDER,
    precision = 2,
    prefix = "",
  } = options;

  if (!isFiniteNumber(value)) return placeholder;

  const abs = Math.abs(value);
  const sign = value < 0 ? "-" : "";

  for (const step of ABBREVIATIONS) {
    if (abs >= step.threshold) {
      const scaled = abs / step.divisor;
      return `${sign}${prefix}${scaled.toFixed(precision)}${step.suffix}`;
    }
  }

  // Below 1,000 — no abbreviation, just plain number with prefix
  return `${sign}${prefix}${abs.toFixed(precision)}`;
}

/* ---------- Combined "$X / +Y%" delta format ---------- */

export interface DeltaFormatOptions {
  /** Currency for the absolute value. Default "USD". */
  currency?: string;
  /** Currency precision. Default 2. */
  currencyPrecision?: number;
  /** Percent precision. Default 2. */
  percentPrecision?: number;
  /** Separator between the two values. Default " / ". */
  separator?: string;
}

/**
 * Format a paired absolute change and percent change.
 *
 * @example
 *   formatDelta(2841.07, 0.34)
 *   // "+$2,841.07 / +0.34%"
 *
 *   formatDelta(-1234.56, -2.1)
 *   // "-$1,234.56 / -2.10%"
 */
export function formatDelta(
  absoluteChange: number | null | undefined,
  percentChange: number | null | undefined,
  options: DeltaFormatOptions = {},
): string {
  const {
    currency = "USD",
    currencyPrecision = 2,
    percentPrecision = 2,
    separator = " / ",
  } = options;

  const abs = formatCurrency(absoluteChange, {
    currency,
    precision: currencyPrecision,
    signed: true,
  });
  const pct = formatPercent(percentChange, {
    precision: percentPrecision,
    signed: true,
  });

  return `${abs}${separator}${pct}`;
}
```

### Test: `format.test.ts`

```typescript
import { describe, it, expect } from "vitest";
import {
  formatCurrency,
  formatPercent,
  formatNumber,
  formatAbbreviated,
  formatDelta,
} from "./format";

describe("formatCurrency", () => {
  it("formats positive values with thousands separators", () => {
    expect(formatCurrency(847293.14)).toBe("$847,293.14");
  });

  it("formats negatives with leading minus by default", () => {
    expect(formatCurrency(-1234.56)).toBe("-$1,234.56");
  });

  it("uses parentheses in accounting mode", () => {
    expect(formatCurrency(-1234.56, { accounting: true })).toBe("($1,234.56)");
  });

  it("prepends explicit + when signed", () => {
    expect(formatCurrency(2841.07, { signed: true })).toBe("+$2,841.07");
  });

  it("returns placeholder for null/undefined/NaN", () => {
    expect(formatCurrency(null)).toBe("—");
    expect(formatCurrency(undefined)).toBe("—");
    expect(formatCurrency(NaN)).toBe("—");
  });

  it("respects custom precision", () => {
    expect(formatCurrency(1234.5678, { precision: 4 })).toBe("$1,234.5678");
  });
});

describe("formatPercent", () => {
  it("treats input as already scaled by default", () => {
    expect(formatPercent(34.7)).toBe("34.70%");
  });

  it("scales decimal input when scale: 'decimal'", () => {
    expect(formatPercent(0.347, { scale: "decimal" })).toBe("34.70%");
  });

  it("applies signed prefix on positives", () => {
    expect(formatPercent(34.7, { signed: true })).toBe("+34.70%");
  });

  it("preserves leading minus on negatives", () => {
    expect(formatPercent(-22.1, { precision: 1 })).toBe("-22.1%");
  });
});

describe("formatAbbreviated", () => {
  it("uses T for trillions", () => {
    expect(formatAbbreviated(2_400_000_000_000)).toBe("2.40T");
  });

  it("uses M with optional prefix", () => {
    expect(formatAbbreviated(12_400_000, { prefix: "$" })).toBe("$12.40M");
  });

  it("returns plain number below 1000", () => {
    expect(formatAbbreviated(847.32)).toBe("847.32");
  });

  it("preserves sign", () => {
    expect(formatAbbreviated(-1_500_000)).toBe("-1.50M");
  });
});

describe("formatDelta", () => {
  it("composes absolute and percent change with default separator", () => {
    expect(formatDelta(2841.07, 0.34)).toBe("+$2,841.07 / +0.34%");
  });

  it("handles negative deltas", () => {
    expect(formatDelta(-1234.56, -2.1)).toBe("-$1,234.56 / -2.10%");
  });

  it("returns placeholders when inputs are null", () => {
    expect(formatDelta(null, null)).toBe("— / —");
  });
});
```

**Tradeoffs:**
- Formatters are cached in `Map`s keyed by precision/currency. `Intl.NumberFormat` instantiation is expensive enough (~200μs per call) that uncached formatting in a 200-row table at 60fps becomes measurable. The cache is unbounded but bounded by the cardinality of `(currency, precision)` pairs the app actually uses (≤20 in practice).
- `null` placeholder default is a single em-dash. If you want different placeholders in different contexts (e.g. "N/A" in exports, "—" in tables), pass `placeholder` explicitly. Don't change the default — many tests will silently break.
- `formatPercent` defaults to `scale: "percent"` because the Rust computation engine returns percent values pre-scaled (e.g. CAGR is `7.82`, not `0.0782`). If a future provider returns decimals, override per-call rather than flipping the default globally.

---

## 2. Class Name Utility

### File: `packages/@chrdfin/ui/src/lib/utils.ts`

Standard shadcn/ui `cn()` utility. Required by every shadcn-derived component.

```typescript
import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Merge Tailwind class names with conflict resolution.
 *
 * Combines `clsx` (conditional classes) and `tailwind-merge` (resolves
 * conflicting utility classes — `cn("p-2", "p-4")` returns `"p-4"`).
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
```

**Dependencies:** `clsx@^2`, `tailwind-merge@^3` (peer-dep of shadcn/ui Tailwind v4 setup).

---

## 3. Hook: `useThemeColors`

Recharts and other JS chart libraries don't inherit CSS custom properties on stroke/fill props — they need actual color values. This hook reads computed CSS variable values from `:root` and returns them as a memoized object, re-reading whenever the `.dark` class on `<html>` toggles.

### File: `packages/@chrdfin/charts/src/hooks/use-theme-colors.ts`

```typescript
import { useEffect, useState } from "react";

/**
 * The set of CSS custom properties exposed to chart libraries.
 *
 * Keep this list in sync with the `--color-*` declarations in
 * `apps/desktop/src/globals.css`. Adding a new theme token requires:
 *   1. Adding it to `:root` and `.dark` in globals.css
 *   2. Adding it to the `@theme inline` block
 *   3. Adding the key here and in `readThemeColors()`
 */
export interface ThemeColors {
  background: string;
  foreground: string;
  border: string;
  muted: string;
  mutedForeground: string;
  primary: string;
  gain: string;
  loss: string;
  neutral: string;
  warning: string;
  chart1: string;
  chart2: string;
  chart3: string;
  chart4: string;
  chart5: string;
}

/* ---------- Internal: read from CSS ---------- */

function readVar(name: string): string {
  if (typeof window === "undefined") return "";
  const value = getComputedStyle(document.documentElement)
    .getPropertyValue(name)
    .trim();
  // Browsers return "" for unknown vars; flag it loudly in dev so
  // misspelled token names don't silently render as transparent.
  if (!value && import.meta.env.DEV) {
    // eslint-disable-next-line no-console
    console.warn(`[useThemeColors] CSS variable ${name} is unset or empty`);
  }
  return value;
}

function readThemeColors(): ThemeColors {
  return {
    background: readVar("--background"),
    foreground: readVar("--foreground"),
    border: readVar("--border"),
    muted: readVar("--muted"),
    mutedForeground: readVar("--muted-foreground"),
    primary: readVar("--primary"),
    gain: readVar("--gain"),
    loss: readVar("--loss"),
    neutral: readVar("--neutral"),
    warning: readVar("--warning"),
    chart1: readVar("--chart-1"),
    chart2: readVar("--chart-2"),
    chart3: readVar("--chart-3"),
    chart4: readVar("--chart-4"),
    chart5: readVar("--chart-5"),
  };
}

/* ---------- Public hook ---------- */

/**
 * Subscribe to chrdfin's theme color tokens.
 *
 * Returns the current set of resolved CSS custom property values from
 * `:root`. The hook re-reads (and re-renders consumers) whenever the
 * `.dark` class toggles on `<html>`, so charts switch colors atomically
 * with the rest of the UI.
 *
 * Implementation notes:
 *   - Uses MutationObserver on `documentElement` watching only the
 *     `class` attribute. Cheap; observer fires once per theme toggle.
 *   - Reads on mount via `useEffect` rather than `useState` initializer
 *     to avoid hydration mismatches and to ensure fonts/styles have
 *     loaded before reading. Initial render will use empty strings,
 *     which Recharts treats as default colors — visible flash for ~16ms
 *     on first render, acceptable for a desktop app.
 *
 * Edge cases:
 *   - SSR: function-guarded with `typeof window === "undefined"`.
 *     Returns empty strings on the server. (Tauri SPAs don't SSR, but
 *     the guard prevents Vitest jsdom from crashing during component
 *     unit tests.)
 *   - Theme toggle race: if user toggles theme during chart animation,
 *     Recharts re-renders mid-transition. This is desired.
 */
export function useThemeColors(): ThemeColors {
  const [colors, setColors] = useState<ThemeColors>(() => ({
    background: "",
    foreground: "",
    border: "",
    muted: "",
    mutedForeground: "",
    primary: "",
    gain: "",
    loss: "",
    neutral: "",
    warning: "",
    chart1: "",
    chart2: "",
    chart3: "",
    chart4: "",
    chart5: "",
  }));

  useEffect(() => {
    if (typeof window === "undefined") return;

    // Read once on mount.
    setColors(readThemeColors());

    // Observe class attribute on <html> for theme toggles.
    const observer = new MutationObserver((mutations) => {
      for (const m of mutations) {
        if (m.type === "attributes" && m.attributeName === "class") {
          setColors(readThemeColors());
          return;
        }
      }
    });

    observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ["class"],
    });

    return () => observer.disconnect();
  }, []);

  return colors;
}
```

**Tradeoffs:**
- Returning a fresh object every render is fine here (consumers are typically chart components that re-render on theme change anyway). If a consumer downstream is doing reference equality on the returned object, wrap with `useMemo` keyed on individual fields — but this hasn't been needed in practice.
- One global observer per hook usage is wasteful at scale. Phase 4+ may consolidate via a top-level `<ThemeColorsProvider>` reading once and supplying via context. For Phase 0, individual observers are fine — chart count is bounded.
- Reading via `getComputedStyle` works regardless of whether the original CSS used hex, OKLCH, or HSL. Recharts accepts any valid CSS color string.

---

## 4. Hook: `useTauriCommand`

Thin wrapper over `@tauri-apps/api/core` `invoke()` that returns a TanStack Query result. All Tauri command consumers go through this — never call `invoke()` directly from a component.

### File: `apps/desktop/src/hooks/use-tauri-command.ts`

```typescript
import { invoke } from "@tauri-apps/api/core";
import {
  useMutation,
  useQuery,
  type UseMutationOptions,
  type UseQueryOptions,
} from "@tanstack/react-query";

/* ---------- Read commands (queries) ---------- */

/**
 * Invoke a Tauri command as a TanStack Query.
 *
 * @param command — Tauri command name (e.g. "health_check", "get_portfolio")
 * @param args    — argument record passed to the command
 * @param options — TanStack Query options (staleTime, enabled, etc.)
 *
 * The query key is `[command, args]` — TanStack will refetch when args
 * change. Pass `enabled: false` to defer until args are ready.
 */
export function useTauriQuery<TResult, TArgs extends Record<string, unknown> = Record<string, never>>(
  command: string,
  args: TArgs = {} as TArgs,
  options?: Omit<UseQueryOptions<TResult, Error>, "queryKey" | "queryFn">,
) {
  return useQuery<TResult, Error>({
    queryKey: [command, args],
    queryFn: async () => {
      try {
        return await invoke<TResult>(command, args);
      } catch (err) {
        // Tauri rejects with a string when the Rust side returns
        // Result::Err. Wrap for consistent error.message access.
        throw err instanceof Error ? err : new Error(String(err));
      }
    },
    ...options,
  });
}

/* ---------- Write commands (mutations) ---------- */

/**
 * Invoke a Tauri command as a TanStack Mutation.
 *
 * @param command — Tauri command name (e.g. "set_setting", "create_portfolio")
 * @param options — TanStack Mutation options (onSuccess, onError, etc.)
 */
export function useTauriMutation<TResult, TArgs extends Record<string, unknown>>(
  command: string,
  options?: UseMutationOptions<TResult, Error, TArgs>,
) {
  return useMutation<TResult, Error, TArgs>({
    mutationFn: async (args) => {
      try {
        return await invoke<TResult>(command, args);
      } catch (err) {
        throw err instanceof Error ? err : new Error(String(err));
      }
    },
    ...options,
  });
}
```

**Tradeoffs:**
- Wrapping `invoke()` in a hook costs one extra function call but gives us: consistent error wrapping, query cache deduplication, automatic refetching on args change, and type safety. The cost is invisible.
- `args` defaults to `{}` for commands with no arguments. TypeScript's default generic resolution gives `Record<string, never>` which is correct.
- The query key includes `args`. Don't pass functions or non-serializable values in args — they break cache key equality. Tauri's wire format requires JSON anyway, so this is enforced naturally.

---

## 5. Hook: `useMarketStatus`

Determines the current state of the US equities market (NYSE/NASDAQ regular hours). Used by the `<MarketStatusIndicator>` in the platform shell header.

### File: `apps/desktop/src/hooks/use-market-status.ts`

```typescript
import { useEffect, useState } from "react";

export type MarketStatus =
  | "open"          // 09:30 - 16:00 ET, weekday, non-holiday
  | "pre-market"    // 04:00 - 09:30 ET, weekday, non-holiday
  | "after-market"  // 16:00 - 20:00 ET, weekday, non-holiday
  | "closed"        // Outside any session window, weekday
  | "weekend"       // Saturday or Sunday
  | "holiday";      // NYSE-observed market holiday

/* ---------- Holiday calendar ---------- */

/**
 * NYSE full-day market holidays. Update annually each December.
 *
 * Format: ISO date (YYYY-MM-DD) in America/New_York interpretation.
 *
 * Note: this list does NOT include early-close days (1pm ET on day
 * after Thanksgiving, July 3 when July 4 falls on Saturday, etc.).
 * Phase 8 calendar feature will introduce a more granular schedule.
 * For Phase 0 the indicator's holiday distinction is best-effort.
 *
 * Source: https://www.nyse.com/markets/hours-calendars
 */
const NYSE_HOLIDAYS_2026: ReadonlyArray<string> = [
  "2026-01-01", // New Year's Day
  "2026-01-19", // MLK Day
  "2026-02-16", // Presidents Day
  "2026-04-03", // Good Friday
  "2026-05-25", // Memorial Day
  "2026-06-19", // Juneteenth
  "2026-07-03", // Independence Day (observed)
  "2026-09-07", // Labor Day
  "2026-11-26", // Thanksgiving
  "2026-12-25", // Christmas
];

const NYSE_HOLIDAYS_2027: ReadonlyArray<string> = [
  "2027-01-01",
  "2027-01-18",
  "2027-02-15",
  "2027-03-26", // Good Friday
  "2027-05-31",
  "2027-06-18", // Juneteenth observed (Sat)
  "2027-07-05", // Independence Day observed (Sun)
  "2027-09-06",
  "2027-11-25",
  "2027-12-24", // Christmas observed (Sat)
];

const HOLIDAYS = new Set<string>([
  ...NYSE_HOLIDAYS_2026,
  ...NYSE_HOLIDAYS_2027,
]);

/* ---------- Status calculation ---------- */

interface EtClock {
  /** ISO date in ET (YYYY-MM-DD) */
  date: string;
  /** Day of week in ET. 0 = Sunday, 6 = Saturday. */
  weekday: number;
  /** Minutes since midnight in ET. */
  minutesSinceMidnight: number;
}

function getEtClock(now: Date): EtClock {
  // Use Intl.DateTimeFormat with America/New_York to extract ET wall
  // time independent of the host machine's timezone. This is the only
  // correct way to compare against NYSE session hours.
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: "America/New_York",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    weekday: "short",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).formatToParts(now);

  const get = (type: string) =>
    parts.find((p) => p.type === type)?.value ?? "";

  const year = get("year");
  const month = get("month");
  const day = get("day");
  const hour = parseInt(get("hour"), 10);
  // Intl returns "24" for midnight in some locales; normalize.
  const normalizedHour = hour === 24 ? 0 : hour;
  const minute = parseInt(get("minute"), 10);
  const weekdayShort = get("weekday"); // "Mon", "Tue", ...

  const weekdayMap: Record<string, number> = {
    Sun: 0, Mon: 1, Tue: 2, Wed: 3, Thu: 4, Fri: 5, Sat: 6,
  };

  return {
    date: `${year}-${month}-${day}`,
    weekday: weekdayMap[weekdayShort] ?? 0,
    minutesSinceMidnight: normalizedHour * 60 + minute,
  };
}

const MARKET_OPEN = 9 * 60 + 30;    // 09:30 ET
const MARKET_CLOSE = 16 * 60;        // 16:00 ET
const PRE_MARKET_START = 4 * 60;     // 04:00 ET
const AFTER_MARKET_END = 20 * 60;    // 20:00 ET

export function computeMarketStatus(now: Date): MarketStatus {
  const clock = getEtClock(now);

  if (clock.weekday === 0 || clock.weekday === 6) return "weekend";
  if (HOLIDAYS.has(clock.date)) return "holiday";

  const m = clock.minutesSinceMidnight;
  if (m >= MARKET_OPEN && m < MARKET_CLOSE) return "open";
  if (m >= PRE_MARKET_START && m < MARKET_OPEN) return "pre-market";
  if (m >= MARKET_CLOSE && m < AFTER_MARKET_END) return "after-market";
  return "closed";
}

/* ---------- Hook ---------- */

export interface UseMarketStatusResult {
  status: MarketStatus;
  /** Current ET clock as "HH:MM" for display next to the indicator. */
  etTime: string;
}

/**
 * Subscribe to market status with a 1-second tick.
 *
 * The interval is intentionally 1Hz — the indicator displays a clock
 * and the user expects second-by-second updates. CPU cost is negligible
 * (one `Date.now()`, one `Intl.DateTimeFormat.formatToParts`).
 */
export function useMarketStatus(): UseMarketStatusResult {
  const [now, setNow] = useState<Date>(() => new Date());

  useEffect(() => {
    const id = window.setInterval(() => setNow(new Date()), 1000);
    return () => window.clearInterval(id);
  }, []);

  const status = computeMarketStatus(now);
  const etTime = new Intl.DateTimeFormat("en-US", {
    timeZone: "America/New_York",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
  }).format(now);

  return { status, etTime };
}
```

### Test: `use-market-status.test.ts`

```typescript
import { describe, it, expect } from "vitest";
import { computeMarketStatus } from "./use-market-status";

// Helper: build a Date that, when interpreted in ET, lands at a
// specific local time. Construct via UTC then offset by 5h (EST) or
// 4h (EDT). For test determinism we use winter dates (EST = UTC-5).
function etDate(iso: string): Date {
  // iso = "2026-03-09T10:30" — interpret as ET wall time
  // Convert to UTC by adding 5h (ET is UTC-5 in winter, UTC-4 in summer)
  // Use a date library or hardcode for tests; here we use March 9 (still EST).
  return new Date(`${iso}:00-05:00`);
}

describe("computeMarketStatus", () => {
  it("returns 'open' during regular hours on a weekday", () => {
    expect(computeMarketStatus(etDate("2026-03-09T10:30"))).toBe("open");
    expect(computeMarketStatus(etDate("2026-03-09T15:59"))).toBe("open");
  });

  it("returns 'pre-market' during 04:00-09:30 ET", () => {
    expect(computeMarketStatus(etDate("2026-03-09T08:00"))).toBe("pre-market");
  });

  it("returns 'after-market' during 16:00-20:00 ET", () => {
    expect(computeMarketStatus(etDate("2026-03-09T18:30"))).toBe("after-market");
  });

  it("returns 'closed' outside all sessions on a weekday", () => {
    expect(computeMarketStatus(etDate("2026-03-09T22:00"))).toBe("closed");
    expect(computeMarketStatus(etDate("2026-03-09T03:30"))).toBe("closed");
  });

  it("returns 'weekend' on Saturday and Sunday", () => {
    // 2026-03-07 is a Saturday
    expect(computeMarketStatus(etDate("2026-03-07T10:30"))).toBe("weekend");
    expect(computeMarketStatus(etDate("2026-03-08T10:30"))).toBe("weekend");
  });

  it("returns 'holiday' on NYSE-observed holidays", () => {
    expect(computeMarketStatus(etDate("2026-01-01T10:30"))).toBe("holiday");
    expect(computeMarketStatus(etDate("2026-12-25T10:30"))).toBe("holiday");
  });
});
```

**Tradeoffs:**
- Holiday calendar is hardcoded by year. Adding a new year is a one-line change in this file. Long-term, sourcing from FRED's `USNYSE` calendar via Tauri command would auto-update, but for Phase 0 this is overkill.
- Early-close days (1pm ET) are NOT modeled. The indicator will say "open" until 4pm even on early-close days, then say "after-market" from 1pm-4pm in reality. This bug is acceptable for Phase 0; Phase 8 (Calendar) will replace this with a richer schedule.
- DST is handled correctly because `Intl.DateTimeFormat` with `timeZone: "America/New_York"` resolves automatically. No manual offset math needed.

---

## 6. `<DeltaValue>`

Renders a number with gain/loss/neutral tinting. The single source of truth for "what color is +0.34%?". Used in every numeric cell that represents a change.

### File: `packages/@chrdfin/ui/src/components/delta-value.tsx`

```typescript
import type { ReactNode } from "react";
import { cn } from "../lib/utils";
import {
  formatCurrency,
  formatPercent,
  formatNumber,
  type CurrencyOptions,
  type PercentOptions,
  type NumberFormatOptions,
} from "../lib/format";

/* ---------- Tint resolution ---------- */

export type DeltaTint = "gain" | "loss" | "neutral";

export interface TintOptions {
  /**
   * Threshold for "neutral" treatment. Values whose absolute value is
   * less than `epsilon` are treated as zero. Default: 0 (strict).
   *
   * Useful for noisy data (e.g. -0.0001% rounding artifacts that should
   * display in neutral, not loss). Set to e.g. 0.005 in percent contexts.
   */
  epsilon?: number;
  /**
   * Force a specific tint regardless of value. Used when the semantic
   * meaning of the value differs from its sign (e.g. a "drawdown"
   * column where -32% is data, not a loss-against-yourself).
   */
  override?: DeltaTint;
}

export function resolveTint(
  value: number | null | undefined,
  options: TintOptions = {},
): DeltaTint {
  if (options.override) return options.override;
  if (typeof value !== "number" || !Number.isFinite(value)) return "neutral";
  const epsilon = options.epsilon ?? 0;
  if (Math.abs(value) <= epsilon) return "neutral";
  return value > 0 ? "gain" : "loss";
}

const TINT_CLASS: Record<DeltaTint, string> = {
  gain: "text-gain",
  loss: "text-loss",
  neutral: "text-neutral",
};

/* ---------- Components ---------- */

export interface DeltaValueProps extends TintOptions {
  value: number | null | undefined;
  /**
   * How to format the value. Determines numeric output and signed prefix.
   */
  format: "currency" | "percent" | "number";
  /** Format options forwarded to the chosen formatter. */
  formatOptions?: CurrencyOptions | PercentOptions | NumberFormatOptions;
  /** Optional content rendered after the value (e.g. unit label). */
  suffix?: ReactNode;
  className?: string;
}

/**
 * Single numeric value with gain/loss/neutral tinting.
 *
 * @example
 *   <DeltaValue value={0.34} format="percent" />
 *   // <span class="text-gain font-mono tabular-nums">+0.34%</span>
 *
 *   <DeltaValue value={-1234.56} format="currency" override="neutral" />
 *   // Renders -$1,234.56 in neutral color (e.g. drawdown column)
 */
export function DeltaValue({
  value,
  format,
  formatOptions = {},
  suffix,
  epsilon,
  override,
  className,
}: DeltaValueProps): JSX.Element {
  // Sign discipline: signed=true is the default for delta displays.
  const optionsWithSign = { signed: true, ...formatOptions };

  const formatted =
    format === "currency"
      ? formatCurrency(value, optionsWithSign as CurrencyOptions)
      : format === "percent"
      ? formatPercent(value, optionsWithSign as PercentOptions)
      : formatNumber(value, optionsWithSign as NumberFormatOptions);

  const tint = resolveTint(value, { epsilon, override });

  return (
    <span
      className={cn(
        "font-mono tabular-nums",
        TINT_CLASS[tint],
        className,
      )}
    >
      {formatted}
      {suffix}
    </span>
  );
}

/* ---------- Combined "+$2,841.07 / +0.34%" variant ---------- */

export interface DeltaPairProps extends TintOptions {
  absoluteValue: number | null | undefined;
  percentValue: number | null | undefined;
  currency?: string;
  currencyPrecision?: number;
  percentPrecision?: number;
  separator?: string;
  className?: string;
}

/**
 * Combined absolute + percent change display.
 *
 * Both values share a single tint determined from `percentValue`
 * (since absolute and percent should never disagree on sign for the
 * same delta — if they do, that's a data bug, not a display concern).
 *
 * @example
 *   <DeltaPair absoluteValue={2841.07} percentValue={0.34} />
 *   // <span class="text-gain ...">+$2,841.07 / +0.34%</span>
 */
export function DeltaPair({
  absoluteValue,
  percentValue,
  currency = "USD",
  currencyPrecision = 2,
  percentPrecision = 2,
  separator = " / ",
  epsilon,
  override,
  className,
}: DeltaPairProps): JSX.Element {
  const abs = formatCurrency(absoluteValue, {
    currency,
    precision: currencyPrecision,
    signed: true,
  });
  const pct = formatPercent(percentValue, {
    precision: percentPrecision,
    signed: true,
  });
  const tint = resolveTint(percentValue, { epsilon, override });

  return (
    <span
      className={cn(
        "font-mono tabular-nums",
        TINT_CLASS[tint],
        className,
      )}
    >
      {abs}
      {separator}
      {pct}
    </span>
  );
}
```

**Tradeoffs:**
- Two components instead of one polymorphic component. The two have different prop shapes and different formatting paths; merging them adds branching without saving lines. Discrete components are easier to read in JSX.
- Default `signed: true` on delta displays. This is the core invariant of the component — if you don't want a leading `+`, use plain `formatCurrency()` directly, not `<DeltaValue>`.
- `override` exists because semantic context sometimes diverges from sign. A drawdown chart's bars are inherently negative but rendering them all in `text-loss` is visually noisy. Use `override="neutral"` and let the chart fill itself convey "drawdown" via shape.

---

## 7. `<MetricsStrip>`

Horizontal row of label/value pairs. Replaces the "row of cards" pattern with a typography-only layout.

### File: `packages/@chrdfin/ui/src/components/metrics-strip.tsx`

```typescript
import type { ReactNode } from "react";
import { cn } from "../lib/utils";
import { DeltaValue, type DeltaValueProps } from "./delta-value";

/* ---------- Single metric ---------- */

export interface MetricProps {
  label: string;
  /** Static value content. Mutually exclusive with `delta`. */
  value?: ReactNode;
  /**
   * Render the value as a tinted DeltaValue. Mutually exclusive with `value`.
   * Provide all DeltaValue props except className.
   */
  delta?: Omit<DeltaValueProps, "className">;
  /** Additional muted qualifier text below the value (e.g. "since inception"). */
  qualifier?: ReactNode;
  className?: string;
}

export function Metric({
  label,
  value,
  delta,
  qualifier,
  className,
}: MetricProps): JSX.Element {
  if ((value === undefined) === (delta === undefined)) {
    // Both or neither — invariant violation
    throw new Error(
      "Metric: exactly one of `value` or `delta` must be provided",
    );
  }

  return (
    <div className={cn("flex flex-col gap-0.5", className)}>
      <span className="text-xs text-muted-foreground uppercase tracking-wide">
        {label}
      </span>
      <span className="text-md font-mono tabular-nums leading-tight">
        {delta ? <DeltaValue {...delta} /> : value}
      </span>
      {qualifier ? (
        <span className="text-xs text-muted-foreground">{qualifier}</span>
      ) : null}
    </div>
  );
}

/* ---------- Strip container ---------- */

export interface MetricsStripProps {
  children: ReactNode;
  /**
   * Layout mode. "flex" distributes children with consistent gap;
   * "grid" forces equal column widths across all metrics.
   * Default "grid".
   */
  layout?: "flex" | "grid";
  /** Border treatment. Default "bottom" (single underline). */
  border?: "none" | "bottom" | "y";
  className?: string;
}

/**
 * Container for a row of metrics.
 *
 * @example
 *   <MetricsStrip>
 *     <Metric label="CAGR" delta={{ value: 7.82, format: "percent" }} />
 *     <Metric label="Total Return" delta={{ value: 312.4, format: "percent" }} />
 *     <Metric label="Max DD" delta={{ value: -32.1, format: "percent", override: "neutral" }} />
 *     <Metric label="Sharpe" value="0.71" />
 *   </MetricsStrip>
 */
export function MetricsStrip({
  children,
  layout = "grid",
  border = "bottom",
  className,
}: MetricsStripProps): JSX.Element {
  const borderClass = {
    none: "",
    bottom: "border-b border-border",
    y: "border-y border-border",
  }[border];

  const layoutClass =
    layout === "grid"
      ? "grid grid-flow-col auto-cols-fr gap-6"
      : "flex flex-wrap gap-6";

  return (
    <div
      className={cn(
        "px-6 py-3 bg-background",
        borderClass,
        layoutClass,
        className,
      )}
    >
      {children}
    </div>
  );
}
```

**Tradeoffs:**
- `grid grid-flow-col auto-cols-fr` gives equal-width columns regardless of content length. This is the right default for backtesting results (8 metrics, all roughly the same width). For variable-width metrics (portfolio dashboard with one long "Total Value" cell), use `layout="flex"`.
- `Metric` throws on missing `value`/`delta`. This is a development-time invariant — production should never see it. Alternative: render a placeholder; rejected because silent fallbacks hide bugs.
- No padding on `Metric` itself; spacing is owned by the parent `MetricsStrip`. This keeps the spacing model in one place.

---

## 8. `<DataTable>`

The most-used component in the application. Built on TanStack Table v8 with shadcn/ui table primitives. Supports sorting, sticky header, density variants, numeric alignment, gain/loss tinting, and zebra striping.

### File: `packages/@chrdfin/ui/src/components/data-table.tsx`

```typescript
import {
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  useReactTable,
  type ColumnDef,
  type SortingState,
  type Row,
} from "@tanstack/react-table";
import { ChevronDown, ChevronUp } from "lucide-react";
import { useState, type ReactNode } from "react";
import { cn } from "../lib/utils";

/* ---------- Public types ---------- */

export type ColumnAlign = "left" | "right" | "center";

/**
 * Extended column definition with chrdfin-specific metadata.
 *
 * Unlike base TanStack Table columns, ours carry alignment information
 * directly so `<DataTable>` can apply right-alignment to numeric cells
 * automatically without the consumer threading a className everywhere.
 */
export interface DataTableColumnDef<TData> extends ColumnDef<TData> {
  align?: ColumnAlign;
  /** CSS width — px or % string. Pass as a literal, e.g. "120px" or "20%". */
  width?: string;
}

export interface DataTableProps<TData> {
  data: TData[];
  columns: DataTableColumnDef<TData>[];
  /** "default" = 32px rows, "compact" = 28px rows. */
  density?: "default" | "compact";
  /** Whether to apply zebra striping. Default true. */
  zebra?: boolean;
  /** Selection: pass a row ID accessor to enable. */
  getRowId?: (row: TData, index: number) => string;
  /** Currently selected row ID, or null. */
  selectedRowId?: string | null;
  /** Called when a row is clicked. Pass null to disable selection UI. */
  onRowClick?: (row: TData) => void;
  /** Empty-state message. Default "No results." */
  emptyMessage?: ReactNode;
  /** Initial sort state. Uncontrolled — for controlled, pass via state. */
  initialSorting?: SortingState;
  className?: string;
}

/* ---------- Component ---------- */

const ALIGN_CLASS: Record<ColumnAlign, string> = {
  left: "text-left",
  right: "text-right",
  center: "text-center",
};

/**
 * Dense, sortable data table for financial data.
 *
 * @example
 *   const columns: DataTableColumnDef<Holding>[] = [
 *     { accessorKey: "ticker", header: "Ticker", align: "left" },
 *     { accessorKey: "shares", header: "Shares", align: "right",
 *       cell: ({ row }) => formatNumber(row.original.shares, { precision: 0 }) },
 *     { accessorKey: "dayChange", header: "Day Chg", align: "right",
 *       cell: ({ row }) => <DeltaValue value={row.original.dayChange} format="percent" /> },
 *   ];
 *
 *   <DataTable data={holdings} columns={columns} density="compact" />
 */
export function DataTable<TData>({
  data,
  columns,
  density = "default",
  zebra = true,
  getRowId,
  selectedRowId = null,
  onRowClick,
  emptyMessage = "No results.",
  initialSorting = [],
  className,
}: DataTableProps<TData>): JSX.Element {
  const [sorting, setSorting] = useState<SortingState>(initialSorting);

  const table = useReactTable({
    data,
    columns,
    state: { sorting },
    onSortingChange: setSorting,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getRowId,
  });

  const rowHeight = density === "compact" ? "h-7" : "h-8"; // 28px / 32px
  const cellPadding = density === "compact" ? "px-2" : "px-3";

  return (
    <div className={cn("relative w-full overflow-auto", className)}>
      <table className="w-full border-collapse text-sm">
        <thead className="sticky top-0 z-10 bg-card">
          {table.getHeaderGroups().map((headerGroup) => (
            <tr
              key={headerGroup.id}
              className="border-b border-border h-8"
            >
              {headerGroup.headers.map((header) => {
                const col = header.column.columnDef as DataTableColumnDef<TData>;
                const align = col.align ?? "left";
                const canSort = header.column.getCanSort();
                const sortDir = header.column.getIsSorted();

                return (
                  <th
                    key={header.id}
                    style={col.width ? { width: col.width } : undefined}
                    className={cn(
                      cellPadding,
                      ALIGN_CLASS[align],
                      "text-xs font-medium text-muted-foreground uppercase tracking-wide",
                      canSort && "cursor-pointer select-none hover:text-foreground",
                    )}
                    onClick={
                      canSort
                        ? header.column.getToggleSortingHandler()
                        : undefined
                    }
                  >
                    <span className={cn(
                      "inline-flex items-center gap-1",
                      align === "right" && "flex-row-reverse",
                      align === "center" && "justify-center",
                    )}>
                      {flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                      {sortDir === "asc" && (
                        <ChevronUp className="size-3 text-foreground" />
                      )}
                      {sortDir === "desc" && (
                        <ChevronDown className="size-3 text-foreground" />
                      )}
                    </span>
                  </th>
                );
              })}
            </tr>
          ))}
        </thead>
        <tbody>
          {table.getRowModel().rows.length === 0 ? (
            <tr>
              <td
                colSpan={columns.length}
                className="h-16 text-center text-muted-foreground"
              >
                {emptyMessage}
              </td>
            </tr>
          ) : (
            table.getRowModel().rows.map((row, index) => (
              <DataTableRow
                key={row.id}
                row={row}
                index={index}
                rowHeight={rowHeight}
                cellPadding={cellPadding}
                zebra={zebra}
                isSelected={getRowId ? row.id === selectedRowId : false}
                onClick={onRowClick}
              />
            ))
          )}
        </tbody>
      </table>
    </div>
  );
}

/* ---------- Row (extracted for clarity) ---------- */

interface DataTableRowProps<TData> {
  row: Row<TData>;
  index: number;
  rowHeight: string;
  cellPadding: string;
  zebra: boolean;
  isSelected: boolean;
  onClick?: (row: TData) => void;
}

function DataTableRow<TData>({
  row,
  index,
  rowHeight,
  cellPadding,
  zebra,
  isSelected,
  onClick,
}: DataTableRowProps<TData>): JSX.Element {
  const isOdd = index % 2 === 1;

  return (
    <tr
      className={cn(
        rowHeight,
        "border-b border-border/50",
        zebra && isOdd && "bg-muted/40",
        onClick && "cursor-pointer hover:bg-accent",
        isSelected && "bg-accent border-l-2 border-l-primary",
      )}
      onClick={onClick ? () => onClick(row.original) : undefined}
    >
      {row.getVisibleCells().map((cell) => {
        const col = cell.column.columnDef as DataTableColumnDef<TData>;
        const align = col.align ?? "left";
        return (
          <td
            key={cell.id}
            className={cn(
              cellPadding,
              ALIGN_CLASS[align],
              align === "right" && "font-mono tabular-nums",
            )}
          >
            {flexRender(cell.column.columnDef.cell, cell.getContext())}
          </td>
        );
      })}
    </tr>
  );
}
```

**Dependencies:** `@tanstack/react-table@^8`, `lucide-react`.

**Tradeoffs:**
- Right-aligned columns automatically receive `font-mono tabular-nums`. This is the conscious default — every right-aligned column in this app is numeric. If you need a right-aligned non-numeric cell (rare; e.g. currency code suffix), override per-cell.
- Sorting is uncontrolled by default for simplicity. For shareable URL state (e.g. backtest results sortable by year), wire `sorting` and `onSortingChange` through props and persist via TanStack Router search params per CLAUDE.md.
- No virtualization. Phase 0 use cases never exceed ~500 rows visible (holdings ≤100, screener results ≤500). If a future view exceeds 1k rows, swap in `@tanstack/react-virtual` — the API surface here is small enough to extend without breaking changes.
- No pagination. Same reasoning. Pagination on a 200-row holdings table is friction without benefit.
- Selection model is single-row. Multi-row select isn't needed in any current view; adding it later means changing `selectedRowId: string | null` to `selectedRowIds: Set<string>` plus checkbox column — non-breaking for current consumers.

---

## 9. `<Sparkline>`

Tiny inline chart for table rows. No axes, no tooltips, no grid — just the line.

### File: `packages/@chrdfin/charts/src/components/sparkline.tsx`

```typescript
import { Line, LineChart, ResponsiveContainer } from "recharts";
import { useMemo } from "react";
import { useThemeColors } from "../hooks/use-theme-colors";

export interface SparklineProps {
  data: number[];
  width?: number;
  height?: number;
  /**
   * Override automatic gain/loss color resolution.
   * By default, the line is gain-tinted if last >= first, loss-tinted otherwise.
   */
  tint?: "auto" | "gain" | "loss" | "neutral";
  strokeWidth?: number;
  className?: string;
}

/**
 * Tiny inline area-less line chart for table rows.
 *
 * @example
 *   <Sparkline data={[100, 102, 99, 105, 108]} />
 */
export function Sparkline({
  data,
  width = 60,
  height = 20,
  tint = "auto",
  strokeWidth = 1.5,
  className,
}: SparklineProps): JSX.Element | null {
  const colors = useThemeColors();

  // Recharts requires `[{ value: number }, ...]` row shape.
  const chartData = useMemo(
    () => data.map((value, index) => ({ index, value })),
    [data],
  );

  const stroke = useMemo(() => {
    if (tint === "gain") return colors.gain;
    if (tint === "loss") return colors.loss;
    if (tint === "neutral") return colors.neutral;
    // auto
    if (data.length < 2) return colors.neutral;
    return data[data.length - 1] >= data[0] ? colors.gain : colors.loss;
  }, [tint, data, colors]);

  if (data.length === 0) return null;

  return (
    <div
      className={className}
      style={{ width, height }}
      aria-hidden="true"
    >
      <ResponsiveContainer width="100%" height="100%">
        <LineChart
          data={chartData}
          margin={{ top: 1, right: 1, bottom: 1, left: 1 }}
        >
          <Line
            type="linear"
            dataKey="value"
            stroke={stroke}
            strokeWidth={strokeWidth}
            dot={false}
            isAnimationActive={false}
          />
        </LineChart>
      </ResponsiveContainer>
    </div>
  );
}
```

**Dependencies:** `recharts@^2`.

**Tradeoffs:**
- `isAnimationActive={false}` — sparklines in a 200-row screener don't animate. Animation costs frames and adds no signal.
- `aria-hidden="true"` — sparklines are decorative-supplementary; the actual data is in the row's adjacent cells. Screen readers should not announce them.
- `type="linear"` rather than `monotone` or `step`. Linear is the most truthful representation; smoothing implies interpolation that isn't there.
- No min/max scaling props. Recharts auto-fits the data to the viewport, which is correct for the relative-shape signal a sparkline conveys. If you need consistent scaling across rows (rare), use a different component.

---

## 10. `<ThemeProvider>`

Manages the `.dark` class on `<html>` and persists user preference via Tauri commands (DuckDB `settings` table per `database-schema-reference.md`). Replaces the typical `next-themes` pattern, which depends on `localStorage` and is forbidden by CLAUDE.md.

### Tauri command contracts (Rust side, declared here for completeness)

```rust
// apps/desktop/src-tauri/src/commands/system.rs

#[tauri::command]
pub async fn get_theme(state: tauri::State<'_, AppState>) -> Result<String, String> {
    // Returns one of "light" | "dark" | "system". Reads from `settings` table.
    // First-launch default: "dark".
}

#[tauri::command]
pub async fn set_theme(theme: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    // Validates theme is one of "light" | "dark" | "system" then upserts.
}
```

### File: `apps/desktop/src/components/providers/theme-provider.tsx`

```typescript
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useState,
  type ReactNode,
} from "react";
import { useTauriQuery, useTauriMutation } from "@/hooks/use-tauri-command";

/* ---------- Types ---------- */

export type Theme = "light" | "dark" | "system";

export type ResolvedTheme = "light" | "dark";

interface ThemeContextValue {
  /** User's stored preference, including "system". */
  theme: Theme;
  /** Effective theme after resolving "system". */
  resolvedTheme: ResolvedTheme;
  /** Update preference. Persisted via Tauri command. */
  setTheme: (theme: Theme) => void;
  /** True until first read from DuckDB completes. */
  isLoading: boolean;
}

const ThemeContext = createContext<ThemeContextValue | null>(null);

/* ---------- System preference detection ---------- */

function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "dark";
  return window.matchMedia("(prefers-color-scheme: light)").matches
    ? "light"
    : "dark";
}

function resolveTheme(theme: Theme): ResolvedTheme {
  return theme === "system" ? getSystemTheme() : theme;
}

function applyDomTheme(resolved: ResolvedTheme): void {
  const html = document.documentElement;
  if (resolved === "dark") {
    html.classList.add("dark");
  } else {
    html.classList.remove("dark");
  }
}

/* ---------- Provider ---------- */

export interface ThemeProviderProps {
  children: ReactNode;
  /** Theme to use until the persisted preference loads. Default "dark". */
  defaultTheme?: Theme;
}

/**
 * Provides theme state. Mount once at the root of the app, inside
 * the QueryClientProvider so Tauri command hooks function.
 *
 * Behavior:
 *   - On mount, applies `defaultTheme` immediately to avoid flash.
 *   - Asynchronously reads `get_theme` from DuckDB. On success, updates
 *     state and re-applies if different.
 *   - Calling `setTheme()` writes to DuckDB and updates DOM optimistically.
 */
export function ThemeProvider({
  children,
  defaultTheme = "dark",
}: ThemeProviderProps): JSX.Element {
  const [theme, setThemeState] = useState<Theme>(defaultTheme);

  // Load persisted preference.
  const { data: persistedTheme, isLoading } = useTauriQuery<Theme>(
    "get_theme",
    {},
    { staleTime: Infinity, gcTime: Infinity },
  );

  // Sync persisted theme into local state once it arrives.
  useEffect(() => {
    if (persistedTheme && persistedTheme !== theme) {
      setThemeState(persistedTheme);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [persistedTheme]);

  // Apply DOM class whenever the resolved theme changes.
  const resolvedTheme = resolveTheme(theme);
  useEffect(() => {
    applyDomTheme(resolvedTheme);
  }, [resolvedTheme]);

  // Re-resolve on system preference change when in "system" mode.
  useEffect(() => {
    if (theme !== "system") return;
    const mq = window.matchMedia("(prefers-color-scheme: light)");
    const handler = () => applyDomTheme(getSystemTheme());
    mq.addEventListener("change", handler);
    return () => mq.removeEventListener("change", handler);
  }, [theme]);

  // Mutation for persisting changes.
  const setThemeMutation = useTauriMutation<void, { theme: Theme }>(
    "set_theme",
  );

  const setTheme = useCallback(
    (next: Theme) => {
      setThemeState(next); // optimistic
      setThemeMutation.mutate({ theme: next });
    },
    [setThemeMutation],
  );

  return (
    <ThemeContext.Provider
      value={{ theme, resolvedTheme, setTheme, isLoading }}
    >
      {children}
    </ThemeContext.Provider>
  );
}

/* ---------- Hook ---------- */

export function useTheme(): ThemeContextValue {
  const ctx = useContext(ThemeContext);
  if (!ctx) {
    throw new Error("useTheme must be used inside <ThemeProvider>");
  }
  return ctx;
}
```

### Mounting in `apps/desktop/src/App.tsx`

```typescript
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RouterProvider } from "@tanstack/react-router";
import { ThemeProvider } from "@/components/providers/theme-provider";
import { router } from "@/router"; // generated by TanStack Router

const queryClient = new QueryClient({
  defaultOptions: {
    queries: { retry: false, refetchOnWindowFocus: false },
  },
});

export function App(): JSX.Element {
  return (
    <QueryClientProvider client={queryClient}>
      <ThemeProvider defaultTheme="dark">
        <RouterProvider router={router} />
      </ThemeProvider>
    </QueryClientProvider>
  );
}
```

**Tradeoffs:**
- DOM class is applied via `useEffect`, which runs after paint. There is a single-frame flash of the default theme on first mount before DuckDB read returns. Mitigations:
  1. Default to `"dark"` matches typical user preference and matches the prototype.
  2. Apply the class in `index.html` at startup based on a Tauri-injected hint (`<html class="dark">`). This requires extending the Rust `main.rs` to read the setting before `app.run()` and inject a script. Phase 0 keeps the simple version; revisit if the flash becomes a real complaint.
- `staleTime: Infinity, gcTime: Infinity` on the read query — theme doesn't change underneath the user. The mutation invalidates nothing because the optimistic local update is the source of truth post-mount.
- "system" mode requires a `matchMedia` listener. Cleaned up in the effect's teardown, so no leak.

---

## 11. `<MarketStatusIndicator>`

Status dot + label + ET clock. Lives in the platform shell header.

### File: `apps/desktop/src/components/shell/market-status-indicator.tsx`

```typescript
import { useMarketStatus, type MarketStatus } from "@/hooks/use-market-status";
import { cn } from "@chrdfin/ui/lib/utils";

const STATUS_LABEL: Record<MarketStatus, string> = {
  open: "Market Open",
  "pre-market": "Pre-Market",
  "after-market": "After-Market",
  closed: "Market Closed",
  weekend: "Weekend",
  holiday: "Holiday",
};

const STATUS_DOT_CLASS: Record<MarketStatus, string> = {
  open: "bg-gain",
  "pre-market": "bg-warning",
  "after-market": "bg-warning",
  closed: "bg-muted-foreground",
  weekend: "bg-muted-foreground",
  holiday: "bg-muted-foreground",
};

export function MarketStatusIndicator(): JSX.Element {
  const { status, etTime } = useMarketStatus();

  return (
    <div className="flex items-center gap-3 text-xs">
      <div className="flex items-center gap-2">
        <span
          className={cn(
            "size-2 rounded-full",
            STATUS_DOT_CLASS[status],
            status === "open" && "animate-pulse",
          )}
          aria-hidden="true"
        />
        <span className="text-muted-foreground">{STATUS_LABEL[status]}</span>
      </div>
      <span className="font-mono tabular-nums text-muted-foreground">
        {etTime} ET
      </span>
    </div>
  );
}
```

**Tradeoffs:**
- `animate-pulse` only when `open`. A pulsing dot signals "live", and is only meaningful during regular hours. Pre/after-market is "open-ish" but reduced volume — the steady amber dot is the right signal.
- ET label is a static "ET" suffix rather than a tooltip. Power users know what timezone NYSE uses; tooltip overhead is wasted here.
- Status changes don't animate (no transition between open and closed colors). A binary state change is exactly what's wanted — fading would suggest uncertainty.

---

## 12. Platform Shell Composition Sketch

Not a full recipe — the sidebar uses shadcn's `<Sidebar>` primitive whose composition is documented at https://ui.shadcn.com/docs/components/sidebar. This is the abbreviated chrdfin-specific structure.

### `apps/desktop/src/components/shell/sidebar.tsx` (abbreviated)

```typescript
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarGroup,
  SidebarGroupLabel,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@chrdfin/ui/components/sidebar";
import { Link, useLocation } from "@tanstack/react-router";
import {
  Activity, BarChart2, BookOpen, Briefcase, Calculator, Calendar,
  FileText, GitCompare, Layers, LineChart, List, Newspaper,
  PiggyBank, Receipt, Scale, Sigma, Star, TrendingUp,
} from "lucide-react";
import { featureFlags } from "@chrdfin/config";

interface NavItem {
  label: string;
  to: string;
  icon: typeof Activity;
  flag: keyof typeof featureFlags;
}

interface NavSection {
  label: string;
  items: NavItem[];
}

/**
 * Section order: Tracking → Analysis & Tools → Market → Reference.
 * Plural labels (Portfolios, Watchlists, Screeners, Calendars) signal
 * multi-instance domains — see docs/technical-blueprint.md § Multi-Instance Domains.
 */
const SECTIONS: NavSection[] = [
  {
    label: "Tracking",
    items: [
      { label: "Portfolios",   to: "/tracking/portfolio",    icon: Briefcase, flag: "tracker" },
      { label: "Transactions", to: "/tracking/transactions", icon: List,      flag: "tracker" },
      { label: "Watchlists",   to: "/tracking/watchlist",    icon: Star,      flag: "tracker" },
    ],
  },
  {
    label: "Analysis & Tools",
    items: [
      { label: "Backtesting",          to: "/analysis/backtest",             icon: LineChart,  flag: "backtest" },
      { label: "Monte Carlo",          to: "/analysis/monte-carlo",          icon: Sigma,      flag: "monteCarlo" },
      { label: "Optimizer",            to: "/analysis/optimizer",            icon: Activity,   flag: "optimizer" },
      { label: "Allocation Optimizer", to: "/analysis/allocation-optimizer", icon: Scale,      flag: "allocationOptimizer" },
      { label: "Calculators",          to: "/tools/calculators",             icon: Calculator, flag: "calculators" },
      { label: "Compare",              to: "/tools/compare",                 icon: GitCompare, flag: "backtest" },
    ],
  },
  {
    label: "Market",
    items: [
      { label: "Screeners", to: "/market/screener", icon: Layers,    flag: "marketData" },
      { label: "News",      to: "/market/news",     icon: Newspaper, flag: "news" },
      { label: "Calendars", to: "/market/calendar", icon: Calendar,  flag: "news" },
    ],
  },
  {
    label: "Reference",
    items: [
      { label: "Stocks",              to: "/reference/stocks",     icon: TrendingUp, flag: "reference" },
      { label: "Options",             to: "/reference/options",    icon: BarChart2,  flag: "reference" },
      { label: "Retirement Accounts", to: "/reference/retirement", icon: PiggyBank,  flag: "reference" },
      { label: "Estate Planning",     to: "/reference/estate",     icon: FileText,   flag: "reference" },
      { label: "Taxes",               to: "/reference/taxes",      icon: Receipt,    flag: "reference" },
      { label: "Guides",              to: "/reference",            icon: BookOpen,   flag: "reference" },
    ],
  },
];

export function AppSidebar(): JSX.Element {
  const location = useLocation();

  return (
    <Sidebar collapsible="icon">
      <SidebarHeader className="px-3 py-2">
        <span className="font-mono font-semibold text-md">CHRD</span>
      </SidebarHeader>
      <SidebarContent>
        {SECTIONS.map((section) => {
          const visibleItems = section.items.filter(
            (item) => featureFlags[item.flag],
          );
          if (visibleItems.length === 0) return null;
          return (
            <SidebarGroup key={section.label}>
              <SidebarGroupLabel>{section.label}</SidebarGroupLabel>
              <SidebarMenu>
                {visibleItems.map((item) => {
                  const Icon = item.icon;
                  const isActive = location.pathname.startsWith(item.to);
                  return (
                    <SidebarMenuItem key={item.to}>
                      <SidebarMenuButton asChild isActive={isActive}>
                        <Link to={item.to}>
                          <Icon className="size-4" />
                          <span>{item.label}</span>
                        </Link>
                      </SidebarMenuButton>
                    </SidebarMenuItem>
                  );
                })}
              </SidebarMenu>
            </SidebarGroup>
          );
        })}
      </SidebarContent>
      <SidebarFooter />
    </Sidebar>
  );
}
```

**Notes:**
- `featureFlags` is imported from `@chrdfin/config` and is a typed object (`Record<FeatureFlag, boolean>`). Untoggling a flag immediately removes its nav items.
- Active state matches by route prefix (`startsWith`) so child routes (e.g. `/market/ticker/AAPL`) keep `Market` highlighted. Adjust to exact match if a future route taxonomy needs it.
- The icon imports are illustrative — pick the actual Lucide icons that fit per item.

---

## Common Pitfalls & Their Fixes

| Symptom | Cause | Fix |
|---|---|---|
| Theme toggle works but charts stay the wrong color | Recharts read `--chart-1` once at mount and cached it as a literal | Use `useThemeColors()` and pass values as props; never use `var(--chart-1)` in chart fill/stroke |
| Numeric columns drift in width on hover | Different cells using `font-sans` and `font-mono` mixed | Apply `font-mono tabular-nums` at the column level via `align: "right"` or via DataTable's auto-application |
| Sort indicator chevron is the wrong color in dark mode | Hardcoded `text-zinc-400` left over from prototype | Use `text-foreground` (active) and rely on header `text-muted-foreground` (inactive) |
| `<DeltaValue>` shows green for `0` | Default `epsilon: 0` means strict-zero is neutral, but tiny floats like `1e-16` are positive | Pass `epsilon: 0.005` (or appropriate tolerance for the metric) |
| Theme flashes on launch | `useEffect` runs after paint | Accept for Phase 0; revisit if it becomes a real complaint (inject class via Tauri pre-script) |
| Sidebar state lost on app restart | Calling `setSidebarOpen` only updates React state | Persist via Tauri command, same pattern as `<ThemeProvider>` |
| Sparkline shows wrong color when toggling theme | `useThemeColors` not subscribing to class changes | Verify the MutationObserver is attached to `documentElement`, not to `body` |
| `<DataTable>` rows reflow on sort | TanStack Table re-rendering with new keys | Pass `getRowId` so React reuses DOM nodes across sort permutations |
| Tauri command rejects with `[object Object]` in logs | `invoke()` rejected with a non-Error value | Always wrap in `useTauriCommand`/`useTauriMutation` — never call `invoke` directly from components |

---

## Testing Patterns

All test files colocated with source per CLAUDE.md (`foo.ts` → `foo.test.ts`).

### Test setup

```typescript
// vitest.config.ts (in @chrdfin/ui package)
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react";

export default defineConfig({
  plugins: [react()],
  test: {
    environment: "jsdom",
    setupFiles: ["./src/test/setup.ts"],
    globals: false,
  },
});
```

```typescript
// src/test/setup.ts
import "@testing-library/jest-dom/vitest";
```

### Component test example

```typescript
// delta-value.test.tsx
import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { DeltaValue } from "./delta-value";

describe("<DeltaValue>", () => {
  it("applies text-gain class on positive percent", () => {
    render(<DeltaValue value={0.34} format="percent" />);
    const el = screen.getByText("+0.34%");
    expect(el).toHaveClass("text-gain");
  });

  it("applies text-loss class on negative currency", () => {
    render(<DeltaValue value={-1234.56} format="currency" />);
    expect(screen.getByText("-$1,234.56")).toHaveClass("text-loss");
  });

  it("renders neutral when value is zero", () => {
    render(<DeltaValue value={0} format="percent" />);
    expect(screen.getByText("+0.00%")).toHaveClass("text-neutral");
    // Note: signed=true means "+0.00%" appears for zero. If "0.00%"
    // is preferred for zero, override formatOptions.signed locally.
  });

  it("respects override prop", () => {
    render(<DeltaValue value={-32.1} format="percent" override="neutral" />);
    expect(screen.getByText("-32.10%")).toHaveClass("text-neutral");
  });

  it("renders placeholder when value is null", () => {
    render(<DeltaValue value={null} format="currency" />);
    expect(screen.getByText("—")).toHaveClass("text-neutral");
  });
});
```

### Tolerance for numerical tests

Per CLAUDE.md: "Numerical tests must specify tolerance (typically 0.01% for financial calculations)."

```typescript
import { describe, it, expect } from "vitest";

describe("compound annual growth rate", () => {
  it("matches reference implementation within 0.01%", () => {
    const calculated = computeCAGR(initialValue, finalValue, years);
    const expected = 7.82;
    expect(Math.abs(calculated - expected) / expected).toBeLessThan(0.0001);
  });
});
```

---

## What NOT to Do

These reinforce CLAUDE.md and `ui-design-system.md` rules in the implementation context:

- Do NOT pass raw hex strings to chart props. Use `useThemeColors()` and reference `colors.chart1`, etc.
- Do NOT inline color logic in component bodies. Use `<DeltaValue>`/`<DeltaPair>` everywhere a value can be positive/negative.
- Do NOT call `Intl.NumberFormat` directly in component render. Use the formatters from `format.ts` — they cache instances.
- Do NOT use `useTheme()` hook from any third-party library. Only the chrdfin `useTheme()` from `@/components/providers/theme-provider`.
- Do NOT call Tauri's `invoke()` from a component. Always go through `useTauriQuery`/`useTauriMutation`.
- Do NOT add prop drilling for `density` or `theme`. Use the providers/hooks defined here.
- Do NOT add a `colorScheme` prop to `<Sparkline>`. The auto/manual `tint` prop is sufficient. If you find yourself needing a fourth color, add it to the design system first.
- Do NOT compute market status in components — call `useMarketStatus()`.
- Do NOT introduce a 3rd-party "currency input" component. The combination of `<Input>` + `formatCurrency()` on blur covers every use case.
- Do NOT add animations to data tables, metrics strips, or any data-heavy view. Animation is reserved for navigation transitions and dialog open/close. Data should appear instantly.

---

## References

- shadcn/ui Tailwind v4: https://ui.shadcn.com/docs/tailwind-v4
- TanStack Table: https://tanstack.com/table/latest/docs/introduction
- TanStack Query: https://tanstack.com/query/latest/docs/framework/react/overview
- TanStack Router: https://tanstack.com/router/latest
- Recharts API: https://recharts.org/en-US/api
- Tauri v2 commands: https://v2.tauri.app/develop/calling-rust/
- Lucide icons: https://lucide.dev/icons
- IBM Plex (Google Fonts): https://fonts.google.com/specimen/IBM+Plex+Sans
- NYSE holiday calendar: https://www.nyse.com/markets/hours-calendars

---

## Document Maintenance

When adding a new component recipe to this file:
1. Confirm the design system (`ui-design-system.md`) already defines any new tokens it requires; if not, add them there first.
2. Specify the package and file path explicitly in the recipe header.
3. Provide complete typed code — no ellipses, no "// TODO".
4. List dependencies with version requirements.
5. Add at least one `describe` block to the testing patterns section if the component has non-trivial logic.
6. Add an entry to "Common Pitfalls" if there's a known footgun.
7. Update the package boundaries table at the top of this file.

When updating an existing recipe:
1. Bump dependency versions only when the source-of-truth design changes (Tailwind v4 to v5, etc.), never opportunistically.
2. Preserve the `@example` JSDoc — Claude Code agents use these as anchor patterns when generating call sites.
