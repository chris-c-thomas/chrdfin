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

/**
 * First-launch starter equity universe. Mirrored in
 * `apps/desktop/src-tauri/src/sync/orchestrator.rs::STARTER_UNIVERSE`
 * and kept in sync by hand — these are reference data, not an
 * integration boundary.
 */
export const STARTER_UNIVERSE = [
  // Core ETFs
  "SPY",
  "QQQ",
  "IWM",
  "DIA",
  "VTI",
  "VOO",
  "VEA",
  "VWO",
  // Bonds
  "AGG",
  "BND",
  "TLT",
  "IEF",
  "SHY",
  "TIP",
  // Sector + Commodities
  "GLD",
  "SLV",
  "USO",
  "VNQ",
  // Major single names
  "AAPL",
  "MSFT",
  "NVDA",
  "AMZN",
  "GOOGL",
  "META",
  "TSLA",
  "BRK-B",
] as const satisfies readonly string[];

/**
 * Macro series the orchestrator pulls on every full or incremental
 * sync. Mirrored in `sync/orchestrator.rs::DEFAULT_MACRO_SERIES`.
 */
export const DEFAULT_MACRO_SERIES = [
  "treasury_3_mo",
  "treasury_10_y",
  "cpi_yoy",
  "unemployment_rate",
] as const satisfies readonly string[];
