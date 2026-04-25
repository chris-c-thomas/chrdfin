export const APP_NAME = "chrdfin";
export const APP_DESCRIPTION = "Personal Financial Intelligence Platform";

export const DEFAULT_BENCHMARK = "SPY";
export const DEFAULT_RISK_FREE_RATE_SERIES = "DGS3MO";
export const DEFAULT_INFLATION_SERIES = "CPIAUCSL";

export const MARKET_HOURS = {
  open: { hour: 9, minute: 30 },
  close: { hour: 16, minute: 0 },
  timezone: "America/New_York",
} as const;

export const POLLING_INTERVALS = {
  realTimeQuotes: 15_000,
  newsSync: 900_000,
} as const;

export const DATA_LIMITS = {
  maxTickersPerQuery: 50,
  maxTickersPerBatch: 100,
  maxScreenerResults: 500,
  maxBacktestAssets: 50,
  maxMCIterations: 1_000_000,
  maxNewsResults: 100,
} as const;
