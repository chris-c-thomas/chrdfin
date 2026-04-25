import type { ISODateString, UUID } from "./common.js";

/**
 * Portfolio classification.
 *
 * - `backtest`  — historical/test portfolio used by the Backtest engine.
 * - `tracked`   — real holdings the user owns; the active "real money" book.
 * - `model`     — target allocation models the user designs but does not (yet) own.
 * - `watchlist` — a named ticker list with no associated holdings.
 * - `paper`     — paper-trading portfolio simulating live trades against real prices,
 *                 staged for the post-1.0 Trading roadmap (see
 *                 `docs/technical-blueprint.md` § Trading Module).
 *
 * The schema column is a `VARCHAR` so additional types can be appended
 * without a migration; this enum is the canonical client-side contract.
 */
export type PortfolioType = "backtest" | "tracked" | "model" | "watchlist" | "paper";

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
