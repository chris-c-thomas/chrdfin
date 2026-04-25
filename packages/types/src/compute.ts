/** All output metrics from the backtest engine. */
export interface PortfolioMetrics {
  readonly totalReturn: number;
  readonly cagr: number;
  readonly annualizedVolatility: number;
  readonly sharpeRatio: number;
  readonly sortinoRatio: number;
  readonly maxDrawdown: number;
  readonly calmarRatio: number;
  readonly treynorRatio?: number;
  readonly alpha?: number;
  readonly beta?: number;
  readonly rSquared?: number;
  readonly informationRatio?: number;
  readonly skewness: number;
  readonly kurtosis: number;
  readonly bestYear: number;
  readonly worstYear: number;
  readonly var95: number;
  readonly cvar95: number;
  readonly winRate: number;
  readonly ulcerIndex: number;
}

/** Strategy interface for rebalancing extensibility. */
export interface RebalancingStrategy {
  readonly name: string;
  readonly description: string;
  readonly configSchema: unknown;
}

export interface StrategyContext {
  readonly currentDate: string;
  readonly currentWeights: Map<string, number>;
  readonly targetWeights: Map<string, number>;
  readonly portfolioValue: number;
  readonly daysSinceLastRebalance: number;
  readonly priceHistory: Map<string, readonly number[]>;
}
