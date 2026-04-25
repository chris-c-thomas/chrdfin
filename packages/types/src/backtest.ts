import type { ISODateString, UUID } from "./common.js";
import type { PortfolioMetrics } from "./compute.js";
import type { PortfolioAllocation } from "./portfolio.js";

export type RebalanceFrequency = "none" | "monthly" | "quarterly" | "annually";

export interface BacktestConfig {
  readonly tickers: readonly string[];
  readonly weights: readonly number[];
  readonly start: ISODateString;
  readonly end: ISODateString;
  readonly rebalance: RebalanceFrequency;
  readonly initialValue: number;
  readonly reinvestDividends: boolean;
  readonly benchmark?: string;
}

export interface EquityCurvePoint {
  readonly date: ISODateString;
  readonly portfolioValue: number;
  readonly benchmarkValue?: number;
}

export interface DrawdownPoint {
  readonly date: ISODateString;
  readonly drawdown: number;
}

export interface AnnualReturn {
  readonly year: number;
  readonly portfolioReturn: number;
  readonly benchmarkReturn?: number;
}

export interface BacktestResult {
  readonly id: UUID;
  readonly config: BacktestConfig;
  readonly metrics: PortfolioMetrics;
  readonly benchmarkMetrics?: PortfolioMetrics;
  readonly equityCurve: readonly EquityCurvePoint[];
  readonly drawdowns: readonly DrawdownPoint[];
  readonly annualReturns: readonly AnnualReturn[];
  readonly finalAllocations: readonly PortfolioAllocation[];
  readonly computedAt: string;
}
