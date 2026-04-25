# Chart Recipes — chrdfin

## Purpose

Production-ready Recharts implementations for every chart used in chrdfin's reference views and beyond. This file is the implementation contract for `@chrdfin/charts`: every chart in the application is either listed here or composed from primitives listed here.

Every recipe is:

- Production-ready (no pseudocode)
- Strictly typed
- Theme-aware via `useThemeColors()`
- Density-aligned with the Bloomberg-terminal aesthetic (10–11px tick labels, no grid by default, no animations on data updates)
- Performance-tuned (memoized transforms, animation disabled on incremental updates)

**Companion documents:**

- `docs/ui-design-system.md` — color tokens, density, chart container conventions
- `docs/ui-component-recipes.md` — `useThemeColors`, formatters, `<Sparkline>` (table-inline variant)
- `docs/technical-blueprint.md` — system architecture
- `docs/type-definitions-reference.md` — shared types

---

## Package Boundaries

| Recipe | File Path |
|---|---|
| Tooltip primitive | `packages/@chrdfin/charts/src/primitives/chart-tooltip.tsx` |
| Axis tick formatters | `packages/@chrdfin/charts/src/primitives/tick-formatters.ts` |
| Chart constants | `packages/@chrdfin/charts/src/primitives/constants.ts` |
| Data utilities | `packages/@chrdfin/charts/src/lib/data-utils.ts` |
| `<PerformanceArea>` | `packages/@chrdfin/charts/src/components/performance-area.tsx` |
| `<AllocationDonut>` | `packages/@chrdfin/charts/src/components/allocation-donut.tsx` |
| `<AnnualReturnsBar>` | `packages/@chrdfin/charts/src/components/annual-returns-bar.tsx` |
| `<ReturnsHistogram>` | `packages/@chrdfin/charts/src/components/returns-histogram.tsx` |
| `<EquityCurveWithDrawdown>` | `packages/@chrdfin/charts/src/components/equity-curve-with-drawdown.tsx` |
| `<MonteCarloCone>` | `packages/@chrdfin/charts/src/components/monte-carlo-cone.tsx` |

All components export named (per CLAUDE.md). All depend on `recharts@^2`, `@chrdfin/ui`, and the `useThemeColors` hook from `@chrdfin/charts/hooks`.

---

## 1. Foundation: Chart Constants

A single source of truth for margins, font sizes, and stroke widths. Every chart imports from this file rather than hardcoding values inline. Changing a stroke width here updates the entire platform.

### File: `packages/@chrdfin/charts/src/primitives/constants.ts`

```typescript
/**
 * Shared visual constants for chrdfin charts.
 *
 * All Recharts components in @chrdfin/charts import from this file. Do not
 * hardcode stroke widths, font sizes, or margins inline in chart components.
 */

/* ---------- Margins ---------- */

export const CHART_MARGIN_DEFAULT = {
  top: 8,
  right: 16,
  bottom: 8,
  left: 16,
} as const;

export const CHART_MARGIN_COMPACT = {
  top: 4,
  right: 8,
  bottom: 4,
  left: 8,
} as const;

/* ---------- Stroke widths ---------- */

export const STROKE_WIDTH = {
  primary: 1.5,
  secondary: 1.25,
  reference: 1,
  axis: 1,
} as const;

/* ---------- Tick formatting ---------- */

export const TICK_STYLE = {
  fontSize: 10,
  fontFamily: "var(--font-mono)",
} as const;

export const AXIS_LINE_STYLE = {
  stroke: "var(--border)",
  strokeWidth: 1,
} as const;

/* ---------- Crosshair / cursor ---------- */

export const CROSSHAIR_STYLE = {
  stroke: "var(--muted-foreground)",
  strokeWidth: 1,
  strokeDasharray: "2 2",
} as const;

/* ---------- Animation ---------- */

/**
 * Animation defaults. Charts with frequently-changing data (live quotes,
 * filter-driven screener charts) should set isAnimationActive={false} to
 * avoid distracting transitions on every tick.
 *
 * Use animation only on first mount of static views (backtest results
 * pages, completed Monte Carlo runs).
 */
export const ANIMATION_FIRST_MOUNT = {
  isAnimationActive: true,
  animationDuration: 300,
  animationEasing: "ease-out" as const,
};

export const ANIMATION_DISABLED = {
  isAnimationActive: false,
};
```

---

## 2. Foundation: Tick Formatters

Reused across every chart's `tickFormatter` props. Wraps the `format.ts` utilities with chart-axis-appropriate defaults.

### File: `packages/@chrdfin/charts/src/primitives/tick-formatters.ts`

```typescript
import {
  formatAbbreviated,
  formatCurrency,
  formatPercent,
} from "@chrdfin/ui/lib/format";

/* ---------- Currency axis ticks ---------- */

/**
 * Currency tick formatter for chart Y-axes.
 *
 * Switches to abbreviated form (`$2.4T`, `$12.4M`) for values >= $10K
 * to keep tick labels short. Below $10K, uses full notation with
 * zero decimal places.
 *
 * @example
 *   <YAxis tickFormatter={currencyTick} />
 */
export function currencyTick(value: number): string {
  if (!Number.isFinite(value)) return "";
  if (Math.abs(value) >= 10_000) {
    return formatAbbreviated(value, { prefix: "$", precision: 1 });
  }
  return formatCurrency(value, { precision: 0 });
}

/* ---------- Percent axis ticks ---------- */

/**
 * Percent tick formatter. Defaults to 0 decimal places (axis ticks
 * don't need precision; tooltip values do).
 */
export function percentTick(value: number): string {
  return formatPercent(value, { precision: 0 });
}

/**
 * Percent tick formatter with sign prefix. Use for charts where axis
 * crosses zero (drawdown, period returns) to make the sign visible.
 */
export function signedPercentTick(value: number): string {
  return formatPercent(value, { precision: 0, signed: true });
}

/* ---------- Date axis ticks ---------- */

/**
 * Date tick formatter that adapts label granularity to the time span.
 *
 * - >= 5 years of data: year-only ("2020")
 * - 1-5 years: short month+year ("Jan '23")
 * - < 1 year: short month+day ("Jan 15")
 *
 * @param timestamp — ISO date string or Unix ms
 * @param spanYears — total span of the chart in years (caller computes)
 */
export function adaptiveDateTick(
  timestamp: string | number,
  spanYears: number,
): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return "";

  if (spanYears >= 5) {
    return date.getUTCFullYear().toString();
  }
  if (spanYears >= 1) {
    const month = date.toLocaleString("en-US", {
      month: "short",
      timeZone: "UTC",
    });
    const year = date.getUTCFullYear().toString().slice(-2);
    return `${month} '${year}`;
  }
  return date.toLocaleString("en-US", {
    month: "short",
    day: "numeric",
    timeZone: "UTC",
  });
}

/**
 * Year-only date tick formatter — for charts where year markers are
 * always preferred (e.g. annual returns bar chart with 10+ years).
 */
export function yearTick(timestamp: string | number): string {
  const date = new Date(timestamp);
  if (Number.isNaN(date.getTime())) return "";
  return date.getUTCFullYear().toString();
}
```

---

## 3. Foundation: ChartTooltip Primitive

shadcn/ui's default tooltip is too padded and too rounded for chrdfin's density. This is a custom Recharts `<Tooltip content>` component that matches the platform's typographic conventions.

### File: `packages/@chrdfin/charts/src/primitives/chart-tooltip.tsx`

```typescript
import type { TooltipProps } from "recharts";
import type { NameType, ValueType } from "recharts/types/component/DefaultTooltipContent";
import { cn } from "@chrdfin/ui/lib/utils";

/* ---------- Public types ---------- */

export interface ChartTooltipFormatter<TPayload = unknown> {
  /**
   * Receives the raw payload row and returns formatted display strings.
   * Called once per series in the payload.
   */
  (entry: TooltipPayloadEntry<TPayload>): {
    label: string;
    value: string;
    /** Color swatch for this series. Hex/RGB/var() all work. */
    color: string;
  } | null;
}

export interface TooltipPayloadEntry<TPayload = unknown> {
  dataKey: string;
  name?: string;
  value: number | string;
  color?: string;
  /** The full data row for this index. */
  payload: TPayload;
}

export interface ChartTooltipContentProps<TPayload = unknown>
  extends TooltipProps<ValueType, NameType> {
  /**
   * Formatter for the date/x-axis label that appears as the tooltip header.
   * Receives the raw label value (typically the x-axis dataKey value).
   */
  labelFormatter?: (label: string | number) => string;
  /**
   * Per-series row formatter. Return null to skip a series.
   */
  rowFormatter?: ChartTooltipFormatter<TPayload>;
  className?: string;
}

/* ---------- Component ---------- */

/**
 * Custom Recharts tooltip content matching chrdfin's typographic system.
 *
 * @example
 *   <Tooltip
 *     content={
 *       <ChartTooltipContent
 *         labelFormatter={(d) => new Date(d).toLocaleDateString()}
 *         rowFormatter={(entry) => ({
 *           label: entry.name ?? entry.dataKey,
 *           value: formatCurrency(entry.value as number),
 *           color: entry.color ?? "var(--foreground)",
 *         })}
 *       />
 *     }
 *     cursor={CROSSHAIR_STYLE}
 *   />
 */
export function ChartTooltipContent<TPayload = unknown>({
  active,
  payload,
  label,
  labelFormatter,
  rowFormatter,
  className,
}: ChartTooltipContentProps<TPayload>): JSX.Element | null {
  if (!active || !payload || payload.length === 0) return null;

  const headerText = labelFormatter
    ? labelFormatter(label as string | number)
    : String(label);

  const rows = payload
    .map((entry, index) => {
      const typedEntry = entry as unknown as TooltipPayloadEntry<TPayload>;
      const formatted = rowFormatter
        ? rowFormatter(typedEntry)
        : {
            label: typedEntry.name ?? typedEntry.dataKey,
            value: String(typedEntry.value),
            color: typedEntry.color ?? "var(--foreground)",
          };
      if (!formatted) return null;
      return { ...formatted, key: `${typedEntry.dataKey}-${index}` };
    })
    .filter((r): r is NonNullable<typeof r> => r !== null);

  if (rows.length === 0) return null;

  return (
    <div
      className={cn(
        "rounded-sm border border-border bg-popover px-2 py-1.5 shadow-md",
        "min-w-[140px] text-popover-foreground",
        className,
      )}
    >
      <div className="text-xs text-muted-foreground border-b border-border/50 pb-1 mb-1 font-mono">
        {headerText}
      </div>
      <div className="flex flex-col gap-0.5">
        {rows.map((row) => (
          <div
            key={row.key}
            className="flex items-center justify-between gap-3 text-xs"
          >
            <div className="flex items-center gap-1.5 min-w-0">
              <span
                aria-hidden="true"
                className="size-2 shrink-0 rounded-sm"
                style={{ backgroundColor: row.color }}
              />
              <span className="text-muted-foreground truncate">
                {row.label}
              </span>
            </div>
            <span className="font-mono tabular-nums text-foreground">
              {row.value}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
}
```

**Tradeoffs:**

- We could use shadcn's `<ChartTooltip>` primitive instead. It's good but adds extra padding and rounded corners that conflict with chrdfin's terminal density. A custom 30-line component is the better tradeoff.
- The shadow (`shadow-md`) is the one place the design system's "no shadows on internal panels" rule is broken — but tooltips are floating overlays, which the rule explicitly excepts.
- Color swatches are 8px squares, not circles. Squares match Carbon's data visualization legend style better.

---

## 4. Foundation: Data Utilities

Helpers for binning, percentile computation, and other transforms charts need before rendering.

### File: `packages/@chrdfin/charts/src/lib/data-utils.ts`

```typescript
/* ---------- Binning ---------- */

export interface Bin {
  /** Lower bound, inclusive. */
  min: number;
  /** Upper bound, exclusive (or inclusive for the last bin). */
  max: number;
  /** Bin midpoint — used as x-coordinate for histograms. */
  midpoint: number;
  /** Count of values in this bin. */
  count: number;
}

export interface ComputeBinsOptions {
  /** Number of bins. Default 20. */
  binCount?: number;
  /**
   * Force a specific min/max for the binning range. If omitted, uses
   * data min/max. Useful for charts that should align bin edges to
   * round numbers (e.g. -50% to +50% on a returns histogram).
   */
  range?: [number, number];
}

/**
 * Compute equal-width bins from a flat array of numeric values.
 *
 * @example
 *   const bins = computeBins([0.1, 0.3, 0.5, 0.7, 0.9, 1.1, 1.3], { binCount: 7 });
 */
export function computeBins(
  values: number[],
  options: ComputeBinsOptions = {},
): Bin[] {
  const { binCount = 20, range } = options;

  if (values.length === 0 || binCount <= 0) return [];

  const finite = values.filter((v) => Number.isFinite(v));
  if (finite.length === 0) return [];

  const [rangeMin, rangeMax] = range ?? [
    Math.min(...finite),
    Math.max(...finite),
  ];

  // Degenerate case: all values identical
  if (rangeMin === rangeMax) {
    return [
      {
        min: rangeMin,
        max: rangeMax,
        midpoint: rangeMin,
        count: finite.length,
      },
    ];
  }

  const width = (rangeMax - rangeMin) / binCount;
  const bins: Bin[] = Array.from({ length: binCount }, (_, i) => {
    const min = rangeMin + i * width;
    const max = i === binCount - 1 ? rangeMax : min + width;
    return { min, max, midpoint: (min + max) / 2, count: 0 };
  });

  for (const value of finite) {
    if (value < rangeMin || value > rangeMax) continue;
    const idx = Math.min(
      binCount - 1,
      Math.floor((value - rangeMin) / width),
    );
    bins[idx].count++;
  }

  return bins;
}

/* ---------- Percentile bands ---------- */

/**
 * Compute percentile values from a sorted array.
 *
 * @param sorted — array MUST be sorted ascending
 * @param percentiles — values in [0, 1], e.g. [0.05, 0.25, 0.5, 0.75, 0.95]
 */
export function percentilesFromSorted(
  sorted: number[],
  percentiles: number[],
): number[] {
  if (sorted.length === 0) return percentiles.map(() => NaN);

  return percentiles.map((p) => {
    const clamped = Math.max(0, Math.min(1, p));
    const idx = clamped * (sorted.length - 1);
    const lower = Math.floor(idx);
    const upper = Math.ceil(idx);
    if (lower === upper) return sorted[lower];
    const fraction = idx - lower;
    return sorted[lower] * (1 - fraction) + sorted[upper] * fraction;
  });
}

/**
 * Compute time-series percentile bands across simulated paths.
 *
 * Given an array of paths (each path being a series of values at
 * each timestep), returns the percentile values at each timestep.
 *
 * @param paths — paths[pathIndex][timeIndex] → value
 * @param percentiles — e.g. [0.05, 0.25, 0.5, 0.75, 0.95]
 * @returns bands[timeIndex] → record of percentile values
 *
 * @example
 *   const bands = pathPercentilesByStep(paths, [0.05, 0.5, 0.95]);
 *   // bands[0] = { p5: 9800, p50: 10000, p95: 10200 }
 */
export function pathPercentilesByStep(
  paths: ReadonlyArray<ReadonlyArray<number>>,
  percentiles: ReadonlyArray<number>,
): Array<Record<string, number>> {
  if (paths.length === 0 || paths[0].length === 0) return [];

  const stepCount = paths[0].length;
  const result: Array<Record<string, number>> = [];

  for (let t = 0; t < stepCount; t++) {
    const valuesAtStep = paths
      .map((p) => p[t])
      .filter((v) => Number.isFinite(v))
      .sort((a, b) => a - b);

    const ps = percentilesFromSorted(valuesAtStep, [...percentiles]);
    const row: Record<string, number> = {};
    percentiles.forEach((p, i) => {
      const key = `p${Math.round(p * 100)}`;
      row[key] = ps[i];
    });
    result.push(row);
  }

  return result;
}

/* ---------- Drawdown computation (display-side) ---------- */

/**
 * Compute drawdown series from an equity curve.
 *
 * Drawdown at time t is (value_t - peak_so_far) / peak_so_far.
 * Returned values are negative percentages (e.g. -32.1 means -32.1%).
 *
 * NOTE: This is the display-side implementation for visualization.
 * The Rust computation engine produces the canonical drawdown stats
 * for backtest results metrics. Use this only when rendering a chart
 * from a raw equity curve without pre-computed drawdown.
 */
export function computeDrawdownSeries(equity: ReadonlyArray<number>): number[] {
  const result: number[] = [];
  let peak = -Infinity;
  for (const v of equity) {
    if (v > peak) peak = v;
    if (peak <= 0 || !Number.isFinite(peak)) {
      result.push(0);
    } else {
      result.push(((v - peak) / peak) * 100);
    }
  }
  return result;
}
```

### Test: `data-utils.test.ts`

```typescript
import { describe, it, expect } from "vitest";
import {
  computeBins,
  percentilesFromSorted,
  pathPercentilesByStep,
  computeDrawdownSeries,
} from "./data-utils";

describe("computeBins", () => {
  it("buckets values into equal-width bins", () => {
    const bins = computeBins([0, 1, 2, 3, 4, 5, 6, 7, 8, 9], { binCount: 5 });
    expect(bins).toHaveLength(5);
    expect(bins[0].count).toBe(2); // 0, 1
    expect(bins[4].count).toBe(2); // 8, 9
  });

  it("handles all-identical input", () => {
    const bins = computeBins([5, 5, 5], { binCount: 3 });
    expect(bins).toHaveLength(1);
    expect(bins[0].count).toBe(3);
  });

  it("returns empty array on empty input", () => {
    expect(computeBins([])).toEqual([]);
  });

  it("respects explicit range", () => {
    const bins = computeBins([1, 2, 3], { binCount: 4, range: [0, 4] });
    expect(bins).toHaveLength(4);
    expect(bins[0].min).toBe(0);
    expect(bins[3].max).toBe(4);
  });
});

describe("percentilesFromSorted", () => {
  it("returns p50 as median", () => {
    const result = percentilesFromSorted([1, 2, 3, 4, 5], [0.5]);
    expect(result[0]).toBe(3);
  });

  it("interpolates between values", () => {
    const result = percentilesFromSorted([0, 100], [0.5]);
    expect(result[0]).toBeCloseTo(50, 5);
  });
});

describe("computeDrawdownSeries", () => {
  it("returns 0 at the peak", () => {
    const dd = computeDrawdownSeries([100, 110, 120]);
    expect(dd[2]).toBe(0);
  });

  it("returns -10% after a 10% decline from peak", () => {
    const dd = computeDrawdownSeries([100, 110, 99]);
    expect(dd[2]).toBeCloseTo(-10, 5);
  });
});
```

**Tradeoffs:**

- `computeBins` discards values outside the explicit range. If you need outliers in a "tail" bin, pass a wider `range` and let the bins absorb them.
- `pathPercentilesByStep` is O(paths × steps × log paths) due to per-step sorting. For Monte Carlo with 10k paths × 252 steps, that's ~2.5M ops — acceptable on a modern CPU (≤50ms). If profiling shows it's a bottleneck, push the computation into Rust as a Tauri command.
- Drawdown computation is duplicated between Rust (canonical, in `chrdfin-core::stats`) and TypeScript (display). The Rust version is the source of truth for metrics; the TS version is for ad-hoc chart rendering only.

---

## 5. `<PerformanceArea>` — Single-Series Area Chart

Used in the portfolio dashboard's right column for the 12-month performance view. Annotated start/end values, no axis labels, subtle fill under the line.

### File: `packages/@chrdfin/charts/src/components/performance-area.tsx`

```typescript
import { useMemo } from "react";
import {
  Area,
  AreaChart,
  ResponsiveContainer,
  XAxis,
  YAxis,
} from "recharts";
import { useThemeColors } from "../hooks/use-theme-colors";
import {
  ANIMATION_DISABLED,
  CHART_MARGIN_COMPACT,
  STROKE_WIDTH,
} from "../primitives/constants";
import { formatCurrency } from "@chrdfin/ui/lib/format";

/* ---------- Public types ---------- */

export interface PerformancePoint {
  /** ISO date or Unix ms timestamp */
  timestamp: string | number;
  value: number;
}

export interface PerformanceAreaProps {
  data: ReadonlyArray<PerformancePoint>;
  height?: number;
  /**
   * Color tinting:
   * - "auto" (default): gain if last >= first, loss otherwise
   * - "neutral": always primary
   */
  tint?: "auto" | "gain" | "loss" | "neutral";
  /** Show inline start/end value annotations. Default true. */
  showEndpoints?: boolean;
  className?: string;
}

/* ---------- Component ---------- */

/**
 * Compact performance chart for portfolio sidebars and tracker tiles.
 *
 * @example
 *   <PerformanceArea
 *     data={[
 *       { timestamp: "2024-04-01", value: 850000 },
 *       { timestamp: "2025-04-01", value: 847293 },
 *     ]}
 *     height={80}
 *   />
 */
export function PerformanceArea({
  data,
  height = 80,
  tint = "auto",
  showEndpoints = true,
  className,
}: PerformanceAreaProps): JSX.Element | null {
  const colors = useThemeColors();

  const stroke = useMemo(() => {
    if (tint === "gain") return colors.gain;
    if (tint === "loss") return colors.loss;
    if (tint === "neutral") return colors.primary;
    if (data.length < 2) return colors.neutral;
    return data[data.length - 1].value >= data[0].value
      ? colors.gain
      : colors.loss;
  }, [tint, data, colors]);

  if (data.length === 0) return null;

  const startValue = data[0].value;
  const endValue = data[data.length - 1].value;
  const gradientId = useMemo(
    () => `perf-area-${Math.random().toString(36).slice(2, 11)}`,
    [],
  );

  return (
    <div className={className} style={{ height }}>
      <div className="relative h-full w-full">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={[...data]} margin={CHART_MARGIN_COMPACT}>
            <defs>
              <linearGradient id={gradientId} x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor={stroke} stopOpacity={0.18} />
                <stop offset="100%" stopColor={stroke} stopOpacity={0} />
              </linearGradient>
            </defs>
            <XAxis dataKey="timestamp" hide />
            <YAxis hide domain={["dataMin", "dataMax"]} />
            <Area
              type="linear"
              dataKey="value"
              stroke={stroke}
              strokeWidth={STROKE_WIDTH.primary}
              fill={`url(#${gradientId})`}
              dot={false}
              activeDot={false}
              {...ANIMATION_DISABLED}
            />
          </AreaChart>
        </ResponsiveContainer>

        {showEndpoints ? (
          <>
            <div className="absolute left-2 top-0 text-[10px] text-muted-foreground font-mono tabular-nums">
              {formatCurrency(startValue, { precision: 0 })}
            </div>
            <div
              className="absolute right-2 top-0 text-[10px] font-mono tabular-nums"
              style={{ color: stroke }}
            >
              {formatCurrency(endValue, { precision: 0 })}
            </div>
          </>
        ) : null}
      </div>
    </div>
  );
}
```

**Tradeoffs:**

- Gradient ID is per-instance to avoid SVG `<defs>` collisions when multiple charts share a page. The randomness is fine because gradients only need to be unique within the document.
- Endpoint labels are absolutely positioned on top of the chart rather than rendered as Recharts `<Label>` components. Recharts label positioning is finicky for "first/last point only" and the absolute approach is simpler.
- `domain={["dataMin", "dataMax"]}` zoom-fits the data. For dramatic visual differences between similar values (e.g. 1% portfolio range), this is exactly what's wanted — a flat sparkline says "nothing happened" and is misleading. Use `domain={[0, "dataMax"]}` for absolute scale views (rare in this app).

---

## 6. `<AllocationDonut>`

Portfolio dashboard's allocation visualization. 160px diameter, no legend, inline labels around the donut showing sector + percent.

### File: `packages/@chrdfin/charts/src/components/allocation-donut.tsx`

```typescript
import { useMemo } from "react";
import { Cell, Pie, PieChart, ResponsiveContainer } from "recharts";
import { useThemeColors } from "../hooks/use-theme-colors";
import { ANIMATION_DISABLED } from "../primitives/constants";
import { formatPercent } from "@chrdfin/ui/lib/format";
import { cn } from "@chrdfin/ui/lib/utils";

/* ---------- Public types ---------- */

export interface AllocationSlice {
  /** Display label (e.g. "Tech", "Healthcare"). */
  label: string;
  /** Percentage 0–100. Caller is responsible for ensuring slices sum to 100. */
  weight: number;
  /**
   * Optional override color. If omitted, slice color is assigned from
   * the chart palette (`--chart-1` through `--chart-5`) plus theme grays
   * for additional slices.
   */
  color?: string;
}

export interface AllocationDonutProps {
  data: ReadonlyArray<AllocationSlice>;
  /** Outer diameter in pixels. Default 160. */
  size?: number;
  /** Inner radius as fraction of outer radius. Default 0.65. */
  innerRadius?: number;
  /** Optional center label (e.g. "100%" or count). */
  centerLabel?: string;
  className?: string;
}

/* ---------- Component ---------- */

const PALETTE_KEYS = ["chart1", "chart2", "chart3", "chart4", "chart5"] as const;

/**
 * Compact allocation donut with inline labels.
 *
 * No legend by design — labels are positioned around the donut adjacent
 * to their slice, which is more efficient at this size than a separate
 * legend table.
 *
 * @example
 *   <AllocationDonut
 *     data={[
 *       { label: "Tech",       weight: 34 },
 *       { label: "Healthcare", weight: 18 },
 *       { label: "Financials", weight: 14 },
 *     ]}
 *   />
 */
export function AllocationDonut({
  data,
  size = 160,
  innerRadius = 0.65,
  centerLabel,
  className,
}: AllocationDonutProps): JSX.Element | null {
  const colors = useThemeColors();

  const slicesWithColor = useMemo(
    () =>
      data.map((slice, index) => ({
        ...slice,
        color:
          slice.color ??
          colors[PALETTE_KEYS[index % PALETTE_KEYS.length]] ??
          colors.mutedForeground,
      })),
    [data, colors],
  );

  if (slicesWithColor.length === 0) return null;

  const outerRadius = size / 2;
  const innerRadiusPx = outerRadius * innerRadius;

  return (
    <div
      className={cn("relative", className)}
      style={{ width: size, height: size }}
    >
      <ResponsiveContainer width="100%" height="100%">
        <PieChart>
          <Pie
            data={[...slicesWithColor]}
            dataKey="weight"
            nameKey="label"
            cx="50%"
            cy="50%"
            innerRadius={innerRadiusPx}
            outerRadius={outerRadius - 1}
            startAngle={90}
            endAngle={-270}
            paddingAngle={1}
            stroke="var(--background)"
            strokeWidth={1}
            {...ANIMATION_DISABLED}
            label={({ cx, cy, midAngle, outerRadius: r, label, weight }) => (
              <SliceLabel
                cx={cx}
                cy={cy}
                midAngle={midAngle}
                outerRadius={r}
                label={label}
                weight={weight}
              />
            )}
            labelLine={false}
          >
            {slicesWithColor.map((slice) => (
              <Cell key={slice.label} fill={slice.color} />
            ))}
          </Pie>
        </PieChart>
      </ResponsiveContainer>

      {centerLabel ? (
        <div className="absolute inset-0 flex items-center justify-center pointer-events-none">
          <span className="text-md font-mono tabular-nums text-foreground">
            {centerLabel}
          </span>
        </div>
      ) : null}
    </div>
  );
}

/* ---------- Inline slice label ---------- */

interface SliceLabelProps {
  cx: number;
  cy: number;
  midAngle: number;
  outerRadius: number;
  label: string;
  weight: number;
}

function SliceLabel({
  cx,
  cy,
  midAngle,
  outerRadius,
  label,
  weight,
}: SliceLabelProps): JSX.Element | null {
  // Hide labels for slices smaller than 4% — too cramped to read.
  if (weight < 4) return null;

  const RADIAN = Math.PI / 180;
  const radius = outerRadius + 12;
  const x = cx + radius * Math.cos(-midAngle * RADIAN);
  const y = cy + radius * Math.sin(-midAngle * RADIAN);

  // Anchor based on which side of the donut we're on.
  const anchor =
    Math.abs(x - cx) < 6 ? "middle" : x > cx ? "start" : "end";

  return (
    <text
      x={x}
      y={y}
      textAnchor={anchor}
      dominantBaseline="middle"
      fontSize={10}
      fontFamily="var(--font-mono)"
      fill="var(--muted-foreground)"
    >
      <tspan x={x} dy="-0.4em" fontWeight={500} fill="var(--foreground)">
        {label}
      </tspan>
      <tspan x={x} dy="1.1em">
        {formatPercent(weight, { precision: 0 })}
      </tspan>
    </text>
  );
}
```

**Tradeoffs:**

- Slices below 4% don't render labels. The cutoff is a usability call — labels for tiny slices overlap and create visual noise. Tooltip hover would expose them, but in this app's terminal aesthetic, hover-tooltips on a 160px donut feel out-of-place. If a slice is too small to show, the data already says "trivial weight."
- Palette wraps with modulo at 5 slices. Allocations with 6+ sectors get repeated colors; for the design's typical 4–6 sectors this is fine. Phase 9 (Optimizer) may need a richer palette — extend `--chart-6` through `--chart-10` in the design system if so.
- `paddingAngle={1}` adds a 1° gap between slices for visual separation. The `stroke="var(--background)"` provides additional seam so slices look "drawn on the surface" rather than fused.
- Slices are not sorted automatically. Caller passes them in display order. For a "largest first" visualization, sort upstream before passing.

---

## 7. `<AnnualReturnsBar>`

Bar chart of yearly returns. Used in the backtest results bottom-left panel and in the portfolio tracker's annual breakdown.

### File: `packages/@chrdfin/charts/src/components/annual-returns-bar.tsx`

```typescript
import { useMemo } from "react";
import {
  Bar,
  BarChart,
  Cell,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useThemeColors } from "../hooks/use-theme-colors";
import {
  ANIMATION_DISABLED,
  AXIS_LINE_STYLE,
  CHART_MARGIN_DEFAULT,
  CROSSHAIR_STYLE,
  TICK_STYLE,
} from "../primitives/constants";
import { ChartTooltipContent } from "../primitives/chart-tooltip";
import { signedPercentTick } from "../primitives/tick-formatters";
import { formatPercent } from "@chrdfin/ui/lib/format";

/* ---------- Public types ---------- */

export interface AnnualReturnsRow {
  year: number;
  /** Percent return (already scaled, e.g. 26.3 = 26.3%) */
  portfolio: number;
  /** Optional benchmark return for the same year. */
  benchmark?: number;
}

export interface AnnualReturnsBarProps {
  data: ReadonlyArray<AnnualReturnsRow>;
  height?: number;
  /** Show benchmark bars alongside portfolio. Default true if data has benchmark. */
  showBenchmark?: boolean;
  className?: string;
}

/* ---------- Component ---------- */

/**
 * Yearly return bars with gain/loss tinting and optional benchmark.
 *
 * @example
 *   <AnnualReturnsBar
 *     data={[
 *       { year: 2020, portfolio: 18.4, benchmark: 16.3 },
 *       { year: 2021, portfolio: 26.3, benchmark: 28.7 },
 *       ...
 *     ]}
 *     height={200}
 *   />
 */
export function AnnualReturnsBar({
  data,
  height = 200,
  showBenchmark,
  className,
}: AnnualReturnsBarProps): JSX.Element | null {
  const colors = useThemeColors();

  const hasBenchmark = data.some((d) => typeof d.benchmark === "number");
  const renderBenchmark = showBenchmark ?? hasBenchmark;

  const dataWithColors = useMemo(
    () =>
      data.map((row) => ({
        ...row,
        portfolioColor: row.portfolio >= 0 ? colors.gain : colors.loss,
      })),
    [data, colors],
  );

  if (data.length === 0) return null;

  return (
    <div className={className} style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={[...dataWithColors]} margin={CHART_MARGIN_DEFAULT}>
          <XAxis
            dataKey="year"
            tick={TICK_STYLE}
            axisLine={AXIS_LINE_STYLE}
            tickLine={false}
            interval={0}
          />
          <YAxis
            tick={TICK_STYLE}
            axisLine={false}
            tickLine={false}
            tickFormatter={signedPercentTick}
            width={36}
          />
          <ReferenceLine y={0} stroke="var(--border)" strokeWidth={1} />
          <Tooltip
            cursor={false}
            content={
              <ChartTooltipContent
                labelFormatter={(year) => `${year}`}
                rowFormatter={(entry) => ({
                  label: entry.dataKey === "portfolio" ? "Portfolio" : "Benchmark",
                  value: formatPercent(entry.value as number, {
                    precision: 2,
                    signed: true,
                  }),
                  color:
                    entry.dataKey === "portfolio"
                      ? (entry.value as number) >= 0
                        ? colors.gain
                        : colors.loss
                      : colors.mutedForeground,
                })}
              />
            }
          />
          <Bar
            dataKey="portfolio"
            {...ANIMATION_DISABLED}
            maxBarSize={32}
            radius={[1, 1, 0, 0]}
          >
            {dataWithColors.map((row) => (
              <Cell key={row.year} fill={row.portfolioColor} />
            ))}
          </Bar>
          {renderBenchmark ? (
            <Bar
              dataKey="benchmark"
              fill={colors.mutedForeground}
              fillOpacity={0.4}
              {...ANIMATION_DISABLED}
              maxBarSize={32}
              radius={[1, 1, 0, 0]}
            />
          ) : null}
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
```

**Tradeoffs:**

- Per-bar coloring via `<Cell>` rather than a single `fill` because each bar's color depends on its sign. Recharts requires `<Cell>` children for variable per-row fills.
- `radius={[1, 1, 0, 0]}` gives a 1px top-radius — barely visible but takes the harshness off the bar tops without crossing into "rounded SaaS chart" territory.
- Benchmark bars are gray and 40% opacity. Putting two colored series side-by-side (portfolio green, benchmark blue) confuses the gain/loss signal. Gray says "reference, not the subject."
- `cursor={false}` on `<Tooltip>` because bar charts don't need a crosshair — hovering a specific bar is unambiguous. Crosshair is for line/area charts where the x-coordinate isn't snapped to a discrete element.

---

## 8. `<ReturnsHistogram>`

Distribution of rolling returns. Used in the backtest results bottom-right panel.

### File: `packages/@chrdfin/charts/src/components/returns-histogram.tsx`

```typescript
import { useMemo } from "react";
import {
  Bar,
  BarChart,
  Cell,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useThemeColors } from "../hooks/use-theme-colors";
import {
  ANIMATION_DISABLED,
  AXIS_LINE_STYLE,
  CHART_MARGIN_DEFAULT,
  TICK_STYLE,
} from "../primitives/constants";
import { ChartTooltipContent } from "../primitives/chart-tooltip";
import { signedPercentTick } from "../primitives/tick-formatters";
import { computeBins } from "../lib/data-utils";
import { formatPercent } from "@chrdfin/ui/lib/format";

/* ---------- Public types ---------- */

export interface ReturnsHistogramProps {
  /**
   * Raw return values (already scaled, e.g. 12.4 = 12.4%). Caller computes
   * rolling returns; this component only handles binning + display.
   */
  values: ReadonlyArray<number>;
  height?: number;
  /** Number of bins. Default 30. */
  binCount?: number;
  /** Optional explicit range; defaults to data min/max. */
  range?: [number, number];
  /** Show a reference line at the mean. Default true. */
  showMean?: boolean;
  className?: string;
}

/* ---------- Component ---------- */

/**
 * Distribution histogram for returns analysis.
 *
 * @example
 *   <ReturnsHistogram
 *     values={twelveMonthRollingReturns}
 *     binCount={30}
 *     range={[-50, 50]}
 *   />
 */
export function ReturnsHistogram({
  values,
  height = 200,
  binCount = 30,
  range,
  showMean = true,
  className,
}: ReturnsHistogramProps): JSX.Element | null {
  const colors = useThemeColors();

  const { bins, mean } = useMemo(() => {
    const computed = computeBins([...values], { binCount, range });
    const finite = values.filter((v) => Number.isFinite(v));
    const computedMean =
      finite.length > 0
        ? finite.reduce((a, b) => a + b, 0) / finite.length
        : 0;
    return { bins: computed, mean: computedMean };
  }, [values, binCount, range]);

  const binsWithColor = useMemo(
    () =>
      bins.map((bin) => ({
        ...bin,
        color: bin.midpoint >= 0 ? colors.gain : colors.loss,
        // Store as scalar fields for Recharts dataKey
        midpointLabel: formatPercent(bin.midpoint, {
          precision: 0,
          signed: true,
        }),
      })),
    [bins, colors],
  );

  if (bins.length === 0) return null;

  return (
    <div className={className} style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <BarChart data={binsWithColor} margin={CHART_MARGIN_DEFAULT}>
          <XAxis
            dataKey="midpoint"
            type="number"
            domain={["dataMin", "dataMax"]}
            tick={TICK_STYLE}
            axisLine={AXIS_LINE_STYLE}
            tickLine={false}
            tickFormatter={signedPercentTick}
            scale="linear"
          />
          <YAxis
            dataKey="count"
            tick={TICK_STYLE}
            axisLine={false}
            tickLine={false}
            width={32}
          />
          {showMean ? (
            <ReferenceLine
              x={mean}
              stroke="var(--foreground)"
              strokeWidth={1}
              strokeDasharray="3 3"
              label={{
                value: `Mean ${formatPercent(mean, { precision: 1, signed: true })}`,
                position: "top",
                fill: "var(--muted-foreground)",
                fontSize: 10,
                fontFamily: "var(--font-mono)",
              }}
            />
          ) : null}
          <Tooltip
            cursor={false}
            content={
              <ChartTooltipContent
                labelFormatter={(midpoint) =>
                  formatPercent(midpoint as number, {
                    precision: 1,
                    signed: true,
                  })
                }
                rowFormatter={(entry) => ({
                  label: "Count",
                  value: String(entry.value),
                  color: "var(--foreground)",
                })}
              />
            }
          />
          <Bar
            dataKey="count"
            fillOpacity={0.6}
            {...ANIMATION_DISABLED}
          >
            {binsWithColor.map((bin) => (
              <Cell key={`${bin.min}-${bin.max}`} fill={bin.color} />
            ))}
          </Bar>
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
```

**Tradeoffs:**

- Bar fillOpacity is 0.6 to soften the histogram color — distribution charts work better at lower saturation than discrete bar charts.
- Bins straddling zero are colored by their midpoint sign. A bin from `-1% to +1%` with midpoint `0%` will tint loss (since `0 >= 0` is gain by our resolveTint logic, and 0 isn't ambiguous here — we fall to gain). For ambiguity-free presentation, pass an even bin count and a symmetric range so no bin crosses zero.
- Type "number" on XAxis is required when `dataKey` is the numeric midpoint. Without it, Recharts treats midpoints as categorical labels and you get unevenly-spaced bars.
- `scale="linear"` is the default but explicit here to make the contract clear.

---

## 9. `<EquityCurveWithDrawdown>` — The Hero Recipe

Two stacked panels with a shared x-axis: equity curve on top, drawdown beneath. Crosshair tracks across both. The most complex chart in the platform.

### File: `packages/@chrdfin/charts/src/components/equity-curve-with-drawdown.tsx`

```typescript
import { useMemo } from "react";
import {
  Area,
  AreaChart,
  Line,
  LineChart,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useThemeColors } from "../hooks/use-theme-colors";
import {
  ANIMATION_FIRST_MOUNT,
  AXIS_LINE_STYLE,
  CHART_MARGIN_DEFAULT,
  CROSSHAIR_STYLE,
  STROKE_WIDTH,
  TICK_STYLE,
} from "../primitives/constants";
import { ChartTooltipContent } from "../primitives/chart-tooltip";
import {
  adaptiveDateTick,
  currencyTick,
  signedPercentTick,
} from "../primitives/tick-formatters";
import {
  formatCurrency,
  formatPercent,
} from "@chrdfin/ui/lib/format";

/* ---------- Public types ---------- */

export interface EquityCurvePoint {
  /** ISO date string. Caller is responsible for sorting ascending. */
  date: string;
  /** Portfolio value at this date. */
  portfolio: number;
  /** Optional benchmark value. */
  benchmark?: number;
  /** Drawdown as percent (negative or zero, e.g. -32.1). */
  drawdown: number;
}

export interface EquityCurveWithDrawdownProps {
  data: ReadonlyArray<EquityCurvePoint>;
  /** Total height of both panels combined. Default 480. */
  height?: number;
  /** Equity panel as fraction of total height. Default 0.7. */
  equityFraction?: number;
  /** Show benchmark dashed line. Default true if data has benchmark. */
  showBenchmark?: boolean;
  /** Optional title rendered above the equity panel. */
  title?: string;
  className?: string;
}

/* ---------- Component ---------- */

const SYNC_ID = "equity-drawdown";

/**
 * Linked equity curve and drawdown chart.
 *
 * Both panels share x-axis ticks and a synchronized hover crosshair via
 * Recharts' `syncId` mechanism. The drawdown panel's x-axis ticks are
 * hidden (the labels appear on the bottom panel instead) but the
 * underlying scale matches.
 *
 * @example
 *   <EquityCurveWithDrawdown
 *     data={backtestResult.timeline}
 *     height={480}
 *     title="Equity Curve"
 *   />
 */
export function EquityCurveWithDrawdown({
  data,
  height = 480,
  equityFraction = 0.7,
  showBenchmark,
  title,
  className,
}: EquityCurveWithDrawdownProps): JSX.Element | null {
  const colors = useThemeColors();

  const hasBenchmark = data.some((d) => typeof d.benchmark === "number");
  const renderBenchmark = showBenchmark ?? hasBenchmark;

  const spanYears = useMemo(() => {
    if (data.length < 2) return 0;
    const first = new Date(data[0].date).getTime();
    const last = new Date(data[data.length - 1].date).getTime();
    return (last - first) / (1000 * 60 * 60 * 24 * 365.25);
  }, [data]);

  const equityHeight = Math.round(height * equityFraction);
  const drawdownHeight = height - equityHeight - 1; // -1 for divider

  if (data.length === 0) return null;

  // Recharts requires plain arrays (not readonly) — clone defensively.
  const chartData = useMemo(() => [...data], [data]);

  return (
    <div
      className={className}
      style={{ height }}
    >
      {title ? (
        <div className="px-4 py-1 text-xs uppercase tracking-wide text-muted-foreground">
          {title}
        </div>
      ) : null}

      {/* ---------- Equity panel ---------- */}
      <div style={{ height: equityHeight }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart
            data={chartData}
            margin={{ ...CHART_MARGIN_DEFAULT, bottom: 0 }}
            syncId={SYNC_ID}
          >
            <XAxis
              dataKey="date"
              tick={false}
              axisLine={AXIS_LINE_STYLE}
              tickLine={false}
              height={1}
            />
            <YAxis
              orientation="right"
              tick={TICK_STYLE}
              axisLine={false}
              tickLine={false}
              tickFormatter={currencyTick}
              width={56}
              domain={["auto", "auto"]}
            />
            <Tooltip
              cursor={CROSSHAIR_STYLE}
              content={
                <ChartTooltipContent
                  labelFormatter={(date) =>
                    new Date(date as string).toLocaleDateString("en-US", {
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                      timeZone: "UTC",
                    })
                  }
                  rowFormatter={(entry) => {
                    if (entry.dataKey === "drawdown") return null; // shown in DD panel
                    const isPortfolio = entry.dataKey === "portfolio";
                    return {
                      label: isPortfolio ? "Portfolio" : "Benchmark",
                      value: formatCurrency(entry.value as number, {
                        precision: 0,
                      }),
                      color: isPortfolio ? colors.chart1 : colors.mutedForeground,
                    };
                  }}
                />
              }
            />
            <Line
              type="monotone"
              dataKey="portfolio"
              stroke={colors.chart1}
              strokeWidth={STROKE_WIDTH.primary}
              dot={false}
              activeDot={{ r: 3, fill: colors.chart1 }}
              {...ANIMATION_FIRST_MOUNT}
            />
            {renderBenchmark ? (
              <Line
                type="monotone"
                dataKey="benchmark"
                stroke={colors.mutedForeground}
                strokeWidth={STROKE_WIDTH.secondary}
                strokeDasharray="4 3"
                dot={false}
                activeDot={false}
                {...ANIMATION_FIRST_MOUNT}
              />
            ) : null}
          </LineChart>
        </ResponsiveContainer>
      </div>

      {/* ---------- Divider ---------- */}
      <div className="h-px bg-border" />

      {/* ---------- Drawdown panel ---------- */}
      <div style={{ height: drawdownHeight }}>
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart
            data={chartData}
            margin={{ ...CHART_MARGIN_DEFAULT, top: 4 }}
            syncId={SYNC_ID}
          >
            <defs>
              <linearGradient id="dd-fill" x1="0" y1="1" x2="0" y2="0">
                <stop offset="0%" stopColor={colors.loss} stopOpacity={0.4} />
                <stop offset="100%" stopColor={colors.loss} stopOpacity={0.05} />
              </linearGradient>
            </defs>
            <XAxis
              dataKey="date"
              tick={TICK_STYLE}
              axisLine={AXIS_LINE_STYLE}
              tickLine={false}
              tickFormatter={(t) => adaptiveDateTick(t, spanYears)}
              minTickGap={48}
            />
            <YAxis
              orientation="right"
              tick={TICK_STYLE}
              axisLine={false}
              tickLine={false}
              tickFormatter={signedPercentTick}
              width={56}
              domain={["dataMin", 0]}
            />
            <ReferenceLine y={0} stroke="var(--border)" strokeWidth={1} />
            <Tooltip
              cursor={CROSSHAIR_STYLE}
              content={
                <ChartTooltipContent
                  labelFormatter={(date) =>
                    new Date(date as string).toLocaleDateString("en-US", {
                      year: "numeric",
                      month: "short",
                      day: "numeric",
                      timeZone: "UTC",
                    })
                  }
                  rowFormatter={(entry) => {
                    if (entry.dataKey !== "drawdown") return null;
                    return {
                      label: "Drawdown",
                      value: formatPercent(entry.value as number, {
                        precision: 2,
                      }),
                      color: colors.loss,
                    };
                  }}
                />
              }
            />
            <Area
              type="monotone"
              dataKey="drawdown"
              stroke={colors.loss}
              strokeWidth={STROKE_WIDTH.primary}
              fill="url(#dd-fill)"
              dot={false}
              activeDot={false}
              {...ANIMATION_FIRST_MOUNT}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </div>
  );
}
```

**Tradeoffs:**

- `syncId` couples the two charts' hover state. This is Recharts' built-in mechanism — when the user moves over the equity panel, the drawdown panel's tooltip activates at the same x-coordinate and vice versa. It's the simplest correct way to do linked-axis charts in Recharts.
- The two panels render as separate `<ResponsiveContainer>`s rather than as a single `<ComposedChart>` because:
  1. They have different y-axis units ($ and %) — `<ComposedChart>` can do dual y-axes but the math gets awkward when the percent axis must always include 0.
  2. The drawdown's `domain={["dataMin", 0]}` constraint is impossible to express alongside an unbounded equity y-axis in a single chart.
  3. Visually separating into two panels matches Bloomberg's PORT screen treatment, which the design system explicitly references.
- Equity panel's x-axis ticks are hidden (`tick={false}`, height=1). The axis line still renders for visual continuity but the labels appear only on the drawdown panel. Saves vertical space.
- `domain={["auto", "auto"]}` on the equity y-axis lets Recharts pick reasonable bounds. For backtests starting at $10K and growing 30x, the tight zoom is correct. If the user needs absolute-scale ($0-baseline) view, a future toggle prop can switch to `[0, "dataMax"]`.
- `activeDot={{ r: 3 }}` on portfolio shows a small dot at the hover position. Benchmark uses `activeDot={false}` because two dots overlap visually. Portfolio is the focal series.
- Drawdown gradient uses a vertical fill with the deeper color at the bottom (where the dips are). The 0.4 → 0.05 alpha range gives soft visual weight without overwhelming the line itself.

---

## 10. `<MonteCarloCone>` — Phase 4 Preview

Phase 4 deliverable. Included here because the recipe is mature enough to lock the API and the design has knock-on effects on `pathPercentilesByStep` already defined in `data-utils.ts`.

### File: `packages/@chrdfin/charts/src/components/monte-carlo-cone.tsx`

```typescript
import { useMemo } from "react";
import {
  Area,
  ComposedChart,
  Line,
  ReferenceLine,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useThemeColors } from "../hooks/use-theme-colors";
import {
  ANIMATION_FIRST_MOUNT,
  AXIS_LINE_STYLE,
  CHART_MARGIN_DEFAULT,
  CROSSHAIR_STYLE,
  STROKE_WIDTH,
  TICK_STYLE,
} from "../primitives/constants";
import { ChartTooltipContent } from "../primitives/chart-tooltip";
import { currencyTick } from "../primitives/tick-formatters";
import { pathPercentilesByStep } from "../lib/data-utils";
import { formatCurrency } from "@chrdfin/ui/lib/format";

/* ---------- Public types ---------- */

export interface MonteCarloConeProps {
  /**
   * Simulated paths. paths[i][t] is the value at time t for simulation i.
   * Caller is responsible for path generation (typically via the Rust
   * `monte_carlo` Tauri command — passing 10k+ paths is acceptable).
   */
  paths: ReadonlyArray<ReadonlyArray<number>>;
  /**
   * Step labels for the x-axis (e.g. ["2025-01-01", "2025-02-01", ...]).
   * Length must match paths[0].length.
   */
  stepLabels: ReadonlyArray<string>;
  /** Initial portfolio value, drawn as a reference line. */
  initialValue?: number;
  /** Total chart height. Default 360. */
  height?: number;
  className?: string;
}

/* ---------- Component ---------- */

const PERCENTILES = [0.05, 0.25, 0.5, 0.75, 0.95] as const;

/**
 * Monte Carlo simulation cone chart.
 *
 * Renders the 5/25/50/75/95 percentile bands as nested filled areas plus
 * the median as a line. Each band uses progressively higher opacity
 * toward the center to convey "more likely" → "more visible."
 *
 * @example
 *   <MonteCarloCone
 *     paths={simulationResult.paths}
 *     stepLabels={simulationResult.steps}
 *     initialValue={100_000}
 *   />
 */
export function MonteCarloCone({
  paths,
  stepLabels,
  initialValue,
  height = 360,
  className,
}: MonteCarloConeProps): JSX.Element | null {
  const colors = useThemeColors();

  const data = useMemo(() => {
    const bands = pathPercentilesByStep(paths, [...PERCENTILES]);
    return bands.map((row, i) => ({
      step: stepLabels[i] ?? String(i),
      ...row,
      // Recharts `Area` stacks need delta values, not absolutes, when
      // using `stackId` semantics. We use absolute values with manual
      // ordering instead — outer band first, inner bands on top.
    }));
  }, [paths, stepLabels]);

  if (data.length === 0) return null;

  return (
    <div className={className} style={{ height }}>
      <ResponsiveContainer width="100%" height="100%">
        <ComposedChart data={data} margin={CHART_MARGIN_DEFAULT}>
          <XAxis
            dataKey="step"
            tick={TICK_STYLE}
            axisLine={AXIS_LINE_STYLE}
            tickLine={false}
            minTickGap={48}
          />
          <YAxis
            orientation="right"
            tick={TICK_STYLE}
            axisLine={false}
            tickLine={false}
            tickFormatter={currencyTick}
            width={56}
            domain={["auto", "auto"]}
          />
          {initialValue !== undefined ? (
            <ReferenceLine
              y={initialValue}
              stroke="var(--muted-foreground)"
              strokeWidth={1}
              strokeDasharray="3 3"
              label={{
                value: `Initial ${formatCurrency(initialValue, { precision: 0 })}`,
                position: "insideTopRight",
                fill: "var(--muted-foreground)",
                fontSize: 10,
                fontFamily: "var(--font-mono)",
              }}
            />
          ) : null}
          <Tooltip
            cursor={CROSSHAIR_STYLE}
            content={
              <ChartTooltipContent
                labelFormatter={(step) => String(step)}
                rowFormatter={(entry) => {
                  const labels: Record<string, string> = {
                    p5: "5th pct",
                    p25: "25th pct",
                    p50: "Median",
                    p75: "75th pct",
                    p95: "95th pct",
                  };
                  const label = labels[entry.dataKey];
                  if (!label) return null;
                  return {
                    label,
                    value: formatCurrency(entry.value as number, {
                      precision: 0,
                    }),
                    color:
                      entry.dataKey === "p50"
                        ? colors.foreground
                        : colors.chart1,
                  };
                }}
              />
            }
          />

          {/* Outer band (5–95) */}
          <Area
            type="monotone"
            dataKey="p95"
            stroke="none"
            fill={colors.chart1}
            fillOpacity={0.1}
            {...ANIMATION_FIRST_MOUNT}
          />
          <Area
            type="monotone"
            dataKey="p5"
            stroke="none"
            fill="var(--background)"
            fillOpacity={1}
            {...ANIMATION_FIRST_MOUNT}
          />

          {/* Inner band (25–75) */}
          <Area
            type="monotone"
            dataKey="p75"
            stroke="none"
            fill={colors.chart1}
            fillOpacity={0.25}
            {...ANIMATION_FIRST_MOUNT}
          />
          <Area
            type="monotone"
            dataKey="p25"
            stroke="none"
            fill="var(--background)"
            fillOpacity={1}
            {...ANIMATION_FIRST_MOUNT}
          />

          {/* Median */}
          <Line
            type="monotone"
            dataKey="p50"
            stroke={colors.foreground}
            strokeWidth={STROKE_WIDTH.primary}
            dot={false}
            activeDot={{ r: 3, fill: colors.foreground }}
            {...ANIMATION_FIRST_MOUNT}
          />
        </ComposedChart>
      </ResponsiveContainer>
    </div>
  );
}
```

**Tradeoffs:**

- Implementing percentile bands with absolute-value `Area`s plus background-color "subtraction" Areas is a Recharts idiom that works around the lack of native ribbon/band support. The alternative would be a custom SVG layer rendered via `<Customized>` — more correct but adds 80+ lines of layout math.
- Median is rendered as a `<Line>` on top of the bands rather than as an `<Area>` so it has clean stroke without fill bleed.
- 5/25/50/75/95 is the standard set. If a future view needs 1/10/50/90/99, add new percentile values to the constant — `pathPercentilesByStep` accepts arbitrary percentiles.
- Path count (10k typical) does not affect render performance because we render percentile bands, not individual paths. The percentile computation is O(paths × steps × log paths) and runs once on data change.
- `domain={["auto", "auto"]}` on the y-axis. For "this could lose 80%" simulations it's important to see the floor; auto-zoom captures that. If the simulation includes the possibility of zero (pension drawdown ruin scenarios), force `[0, "auto"]` via a prop in a future revision.

---

## 11. Empty & Loading States

Charts must handle three states: empty data (no rows), loading (Tauri command in flight), and error (Tauri command rejected). The conventions:

```typescript
// In a route component
function BacktestResultsPage(): JSX.Element {
  const { data, isLoading, error } = useTauriQuery<BacktestResult>(
    "run_backtest",
    { config },
  );

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-96 text-xs text-muted-foreground">
        Running backtest...
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex flex-col items-center justify-center h-96 gap-2 text-xs">
        <span className="text-destructive">Backtest failed</span>
        <span className="text-muted-foreground">{error.message}</span>
      </div>
    );
  }

  if (!data || data.timeline.length === 0) {
    return (
      <div className="flex items-center justify-center h-96 text-xs text-muted-foreground">
        No data available.
      </div>
    );
  }

  return <EquityCurveWithDrawdown data={data.timeline} />;
}
```

**Conventions:**

- No spinners. Text-only loading state ("Running backtest...") matches the platform's terminal aesthetic.
- No illustrations. No empty-state SVGs of clipboards or magnifying glasses.
- Error messages display the actual error from the Rust backend. The Rust side is responsible for translating internal errors into human-readable strings via `thiserror`.
- Loading and error states share the chart's height to prevent layout shift when data resolves.

---

## 12. Common Pitfalls

| Symptom | Cause | Fix |
|---|---|---|
| Chart renders at 0×0 size | Parent container has no height/width | Always wrap in a sized container; don't rely on `<ResponsiveContainer>` to size from nothing |
| Tooltip shows wrong colors after theme toggle | Color values cached in Recharts internal state | Pass colors as props from `useThemeColors()` — never hardcode CSS var strings in stroke/fill |
| `syncId` charts don't synchronize | Different `data` arrays passed to each chart | Pass identical `data` references to both charts; they sync on x-axis values, which must align |
| Histogram bars unevenly spaced | XAxis treating midpoints as categorical | Set `type="number"` on XAxis explicitly |
| Donut labels overlap on small slices | Slices below ~4% need tooltip-only labels | The `SliceLabel` component already filters; if needed, raise the threshold |
| Drawdown panel shows positive values | Caller passing absolute drawdown instead of negative | Drawdown convention is negative %; verify the Rust backend returns signed values |
| Equity curve animation jitters on hover | `activeDot` with large radius causing reflow | Keep `activeDot={{ r: 3 }}` or smaller; never larger |
| Recharts errors "data is not a function" | Passing readonly array directly | Spread to plain array (`[...data]`) before passing — Recharts mutates internally |
| Crosshair line appears at chart edges only | `cursor={false}` set globally on Tooltip | Use `cursor={CROSSHAIR_STYLE}` for line/area charts; `false` only for bar charts |
| Y-axis tick labels truncate | `width` prop on YAxis too narrow for currency labels | Default 56px is enough for `$847.3M`; widen for `$1.234T` axes |
| First render flickers wrong colors | `useThemeColors` returns empty strings on initial mount | Accept the single-frame flicker; chart libraries handle empty strings as defaults |
| Monte Carlo cone bands look gappy | Path count too low for smooth percentiles | Run with ≥1000 paths; 10k is the recommended Phase 4 default |

---

## 13. Testing Patterns

### Chart components: smoke tests + snapshot

Charts are inherently visual; full unit tests are low-leverage. Smoke tests verify the component mounts without throwing and that key labels/values appear.

```typescript
// equity-curve-with-drawdown.test.tsx
import { describe, it, expect } from "vitest";
import { render } from "@testing-library/react";
import { EquityCurveWithDrawdown } from "./equity-curve-with-drawdown";

const sampleData = [
  { date: "2020-01-01", portfolio: 10000, benchmark: 10000, drawdown: 0 },
  { date: "2020-06-01", portfolio: 8500,  benchmark: 9200,  drawdown: -15 },
  { date: "2021-01-01", portfolio: 11200, benchmark: 10800, drawdown: 0 },
  { date: "2024-01-01", portfolio: 14300, benchmark: 13100, drawdown: -5.2 },
];

describe("<EquityCurveWithDrawdown>", () => {
  it("mounts without throwing on valid data", () => {
    const { container } = render(
      <EquityCurveWithDrawdown data={sampleData} title="Equity Curve" />,
    );
    expect(container.querySelector("svg")).toBeTruthy();
  });

  it("renders the title when provided", () => {
    const { getByText } = render(
      <EquityCurveWithDrawdown data={sampleData} title="Equity Curve" />,
    );
    expect(getByText("Equity Curve")).toBeTruthy();
  });

  it("renders nothing on empty data", () => {
    const { container } = render(<EquityCurveWithDrawdown data={[]} />);
    expect(container.firstChild).toBeNull();
  });
});
```

### Data utility tests: full numerical coverage

Data utilities (binning, percentiles, drawdown) deserve thorough testing because their bugs surface as subtle chart misalignment that's hard to catch visually. See `data-utils.test.ts` in section 4.

### Visual regression: out of scope

Visual regression testing (Chromatic, Playwright snapshots) is not part of Phase 0–10. The platform is single-user and personal; visual review during development is sufficient. If chart correctness becomes critical, revisit.

---

## 14. References

- Recharts API: <https://recharts.org/en-US/api>
- Recharts examples: <https://recharts.org/en-US/examples>
- Recharts `syncId` documentation: <https://recharts.org/en-US/api/LineChart#syncId>
- shadcn/ui Chart primitive (for comparison): <https://ui.shadcn.com/docs/components/chart>
- IBM Carbon Charts (alternative library, for reference only): <https://carbondesignsystem.com/data-visualization/getting-started>
- Drawdown definition: <https://www.investopedia.com/terms/d/drawdown.asp>
- Monte Carlo simulation in finance: <https://www.investopedia.com/terms/m/montecarlosimulation.asp>

---

## 15. Document Maintenance

Adding a new chart recipe:

1. Confirm the design system already defines any required tokens. Add new chart palette entries if needed (see `ui-design-system.md` section "Chart series").
2. Add an entry to the package boundaries table.
3. Provide complete typed code with `useThemeColors()` integration.
4. Include `ANIMATION_DISABLED` (frequent updates) or `ANIMATION_FIRST_MOUNT` (static results) on every Recharts shape component.
5. Specify margin via `CHART_MARGIN_DEFAULT` or `CHART_MARGIN_COMPACT` rather than inline.
6. Add at least a smoke test verifying empty/normal data behavior.
7. Add a "Common Pitfalls" entry if the chart has a non-obvious failure mode.

Updating an existing recipe:

1. Stroke widths, font sizes, and colors should change in `constants.ts` or the design system tokens — never inline in a single component.
2. Bumping Recharts major versions requires reviewing all chart components — Recharts has historically broken `syncId` and `dataKey` typing across 1.x → 2.x.
3. New Recharts components (e.g. `Funnel`, `Sankey`) added to the platform get a recipe entry here, not just inline use.
