import type { ISODateString, UUID } from "./common.js";

export type PortfolioType = "backtest" | "tracked" | "model" | "watchlist";

export interface PortfolioAllocation {
  readonly ticker: string;
  readonly weight: number;
}

export interface PortfolioConfig {
  readonly allocations: readonly PortfolioAllocation[];
  readonly rebalance: "none" | "monthly" | "quarterly" | "annually";
  readonly reinvestDividends: boolean;
  readonly initialValue: number;
}

export interface Portfolio {
  readonly id: UUID;
  readonly name: string;
  readonly description?: string;
  readonly portfolioType: PortfolioType;
  readonly config: PortfolioConfig;
  readonly createdAt: string;
  readonly updatedAt: string;
}

export interface PortfolioContext {
  readonly portfolio: Portfolio;
  readonly asOf: ISODateString;
  readonly allocations: readonly PortfolioAllocation[];
  readonly totalValue: number;
}
