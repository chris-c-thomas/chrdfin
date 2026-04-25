import type { ISODateString, UUID } from "./common.js";
import type { PortfolioAllocation } from "./portfolio.js";

export type OptimizationMethod =
  | "mean_variance"
  | "min_volatility"
  | "max_sharpe"
  | "risk_parity"
  | "black_litterman";

export interface OptimizationConstraints {
  readonly minWeight: number;
  readonly maxWeight: number;
  readonly allowShort: boolean;
  readonly targetReturn?: number;
  readonly targetVolatility?: number;
}

export interface OptimizationConfig {
  readonly method: OptimizationMethod;
  readonly tickers: readonly string[];
  readonly historicalStart: ISODateString;
  readonly historicalEnd: ISODateString;
  readonly constraints: OptimizationConstraints;
  readonly riskFreeRate?: number;
}

export interface OptimizationResult {
  readonly id: UUID;
  readonly config: OptimizationConfig;
  readonly allocations: readonly PortfolioAllocation[];
  readonly expectedReturn: number;
  readonly expectedVolatility: number;
  readonly expectedSharpe: number;
  readonly computedAt: string;
}

export interface EfficientFrontierConfig {
  readonly tickers: readonly string[];
  readonly historicalStart: ISODateString;
  readonly historicalEnd: ISODateString;
  readonly points: number;
  readonly constraints: OptimizationConstraints;
}

export interface EfficientFrontierPoint {
  readonly expectedReturn: number;
  readonly expectedVolatility: number;
  readonly expectedSharpe: number;
  readonly allocations: readonly PortfolioAllocation[];
}

export interface EfficientFrontierResult {
  readonly id: UUID;
  readonly config: EfficientFrontierConfig;
  readonly points: readonly EfficientFrontierPoint[];
  readonly maxSharpePoint: EfficientFrontierPoint;
  readonly minVolatilityPoint: EfficientFrontierPoint;
  readonly computedAt: string;
}
