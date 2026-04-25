import type { ISODateString, UUID } from "./common.js";

export type MonteCarloMethod = "parametric_gbm" | "historical_bootstrap" | "block_bootstrap";

export interface MonteCarloConfig {
  readonly method: MonteCarloMethod;
  readonly tickers: readonly string[];
  readonly weights: readonly number[];
  readonly initialValue: number;
  readonly horizonYears: number;
  readonly iterations: number;
  readonly historicalStart?: ISODateString;
  readonly historicalEnd?: ISODateString;
  readonly bootstrapBlockSize?: number;
  readonly expectedReturn?: number;
  readonly volatility?: number;
  readonly inflationAdjusted: boolean;
}

export interface PercentilePath {
  readonly percentile: number;
  readonly values: readonly number[];
}

export interface HistogramBin {
  readonly lower: number;
  readonly upper: number;
  readonly count: number;
}

export interface MonteCarloResult {
  readonly id: UUID;
  readonly config: MonteCarloConfig;
  readonly percentilePaths: readonly PercentilePath[];
  readonly terminalDistribution: readonly HistogramBin[];
  readonly meanTerminalValue: number;
  readonly medianTerminalValue: number;
  readonly probabilityOfLoss: number;
  readonly probabilityOfDoubling: number;
  readonly computedAt: string;
}
