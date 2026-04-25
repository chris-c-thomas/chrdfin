/**
 * Number formatting utilities for chrdfin.
 *
 * All functions are pure and deterministic. They use `Intl.NumberFormat`
 * with explicit locale (`en-US`) so output is identical across machines
 * and OS locale settings.
 *
 * All formatters accept `null`/`undefined` and return a placeholder
 * (default "—"), since financial data has gaps.
 */

export interface FormatOptions {
  placeholder?: string;
  signed?: boolean;
}

export interface CurrencyOptions extends FormatOptions {
  currency?: string;
  precision?: number;
  accounting?: boolean;
}

export interface PercentOptions extends FormatOptions {
  precision?: number;
  scale?: "percent" | "decimal";
}

export interface NumberFormatOptions extends FormatOptions {
  precision?: number;
}

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

const currencyFormatters = new Map<string, Intl.NumberFormat>();

function getCurrencyFormatter(currency: string, precision: number): Intl.NumberFormat {
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
  return applySign(`${formatter.format(scaled)}%`, scaled, signed);
}

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

export function formatNumber(
  value: number | null | undefined,
  options: NumberFormatOptions = {},
): string {
  const { placeholder = DEFAULT_PLACEHOLDER, precision = 2, signed = false } = options;

  if (!isFiniteNumber(value)) return placeholder;
  const formatter = getNumberFormatter(precision);
  return applySign(formatter.format(value), value, signed);
}

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

export function formatAbbreviated(
  value: number | null | undefined,
  options: NumberFormatOptions & { prefix?: string } = {},
): string {
  const { placeholder = DEFAULT_PLACEHOLDER, precision = 2, prefix = "" } = options;

  if (!isFiniteNumber(value)) return placeholder;

  const abs = Math.abs(value);
  const sign = value < 0 ? "-" : "";

  for (const step of ABBREVIATIONS) {
    if (abs >= step.threshold) {
      const scaled = abs / step.divisor;
      return `${sign}${prefix}${scaled.toFixed(precision)}${step.suffix}`;
    }
  }
  return `${sign}${prefix}${abs.toFixed(precision)}`;
}

export interface DeltaFormatOptions {
  currency?: string;
  currencyPrecision?: number;
  percentPrecision?: number;
  separator?: string;
}

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
