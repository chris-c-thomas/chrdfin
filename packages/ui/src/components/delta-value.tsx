import { type HTMLAttributes, type JSX } from "react";

import {
  formatCurrency,
  formatPercent,
  type CurrencyOptions,
  type PercentOptions,
} from "../lib/format.js";
import { cn } from "../lib/utils.js";

export type DeltaFormat = "currency" | "percent" | "raw";

export interface DeltaValueProps extends HTMLAttributes<HTMLSpanElement> {
  value: number | null | undefined;
  format?: DeltaFormat;
  currencyOptions?: CurrencyOptions;
  percentOptions?: PercentOptions;
  /** Show explicit "+" on positive values. Default true. */
  signed?: boolean;
  /** Override displayed text. */
  display?: string;
}

/**
 * Render a numeric value with gain/loss/neutral semantic tinting.
 *
 * Components and pages should always go through DeltaValue (or a thin
 * wrapper) for any value where direction matters. Never set
 * `text-loss`/`text-gain` directly in business logic.
 */
export function DeltaValue({
  value,
  format = "raw",
  currencyOptions,
  percentOptions,
  signed = true,
  display,
  className,
  ...props
}: DeltaValueProps): JSX.Element {
  const tone =
    typeof value === "number" && Number.isFinite(value)
      ? value > 0
        ? "text-gain"
        : value < 0
          ? "text-loss"
          : "text-neutral"
      : "text-muted-foreground";

  const text =
    display ??
    (format === "currency"
      ? formatCurrency(value, { signed, ...currencyOptions })
      : format === "percent"
        ? formatPercent(value, { signed, ...percentOptions })
        : value === null || value === undefined || !Number.isFinite(value)
          ? "—"
          : signed && value > 0
            ? `+${value}`
            : String(value));

  return (
    <span className={cn("font-mono tabular-nums", tone, className)} {...props}>
      {text}
    </span>
  );
}
