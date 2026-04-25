import { z } from "zod";

const isoDate = z.string().regex(/^\d{4}-\d{2}-\d{2}$/, "Must be YYYY-MM-DD");

const tickerSymbol = z
  .string()
  .min(1)
  .max(10)
  .regex(/^[A-Z0-9.-]+$/, "Invalid ticker format");

/**
 * Comma-separated list of tickers, max 50. Encoded as "AAPL,MSFT,GOOGL"
 * and decoded to ["AAPL","MSFT","GOOGL"]. Avoids JSON for arrays so URLs
 * stay short and human-readable.
 */
const tickerList = z
  .string()
  .optional()
  .transform((s) => (s ? s.split(",").filter(Boolean) : []))
  .pipe(z.array(tickerSymbol).max(50));

const stringList = z
  .string()
  .optional()
  .transform((s) => (s ? s.split(",").filter(Boolean) : []))
  .pipe(z.array(z.string().min(1).max(50)).max(50));

const optionalNumber = z
  .string()
  .optional()
  .transform((s) => (s !== undefined && s !== "" ? Number(s) : undefined));

/* ============================================================
   Backtest
   ============================================================ */

export const BacktestSearchSchema = z.object({
  tickers: tickerList.optional(),
  weights: z
    .string()
    .optional()
    .transform((s) =>
      s ? s.split(",").map(Number).filter((n) => !Number.isNaN(n)) : [],
    )
    .pipe(z.array(z.number().min(0).max(1)).max(50)),
  start: isoDate.optional(),
  end: isoDate.optional(),
  rebalance: z
    .enum(["none", "monthly", "quarterly", "annually"])
    .optional()
    .default("annually"),
  initial: optionalNumber.pipe(z.number().positive().optional()),
});

export type BacktestSearch = z.infer<typeof BacktestSearchSchema>;

/* ============================================================
   Monte Carlo
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
    .pipe(z.number().int().min(100).max(1_000_000)),
  horizon: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 30))
    .pipe(z.number().int().min(1).max(60)),
  mu: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 7))
    .pipe(z.number().min(-50).max(50)),
  sigma: z
    .string()
    .optional()
    .transform((s) => (s ? Number(s) : 15))
    .pipe(z.number().min(0).max(100)),
});

export type MonteCarloSearch = z.infer<typeof MonteCarloSearchSchema>;

/* ============================================================
   Portfolio tracker
   ============================================================ */

export const PortfolioSearchSchema = z.object({
  id: z.string().uuid().optional(),
});

export type PortfolioSearch = z.infer<typeof PortfolioSearchSchema>;

/* ============================================================
   Screener
   ============================================================ */

export const ScreenerSearchSchema = z.object({
  asset: z.enum(["stocks", "etfs", "all"]).optional().default("stocks"),
  sectors: stringList.optional(),
  marketCapMin: optionalNumber.pipe(z.number().nonnegative().optional()),
  marketCapMax: optionalNumber.pipe(z.number().positive().optional()),
  yieldMin: optionalNumber.pipe(z.number().nonnegative().max(100).optional()),
  peMin: optionalNumber.pipe(z.number().optional()),
  peMax: optionalNumber.pipe(z.number().optional()),
  sort: z
    .enum(["ticker", "marketCap", "price", "dayChange", "ytd", "yield", "pe", "volume"])
    .optional(),
  dir: z.enum(["asc", "desc"]).optional().default("desc"),
});

export type ScreenerSearch = z.infer<typeof ScreenerSearchSchema>;

/* ============================================================
   News
   ============================================================ */

export const NewsSearchSchema = z.object({
  category: z
    .enum(["all", "earnings", "macro", "policy", "company"])
    .optional()
    .default("all"),
  source: z.string().optional(),
  tickers: tickerList.optional(),
  q: z.string().max(200).optional(),
});

export type NewsSearch = z.infer<typeof NewsSearchSchema>;

/* ============================================================
   Calendar
   ============================================================ */

export const CalendarSearchSchema = z.object({
  view: z.enum(["earnings", "economic", "ipo", "splits"]).optional().default("earnings"),
  from: isoDate.optional(),
  to: isoDate.optional(),
});

export type CalendarSearch = z.infer<typeof CalendarSearchSchema>;
