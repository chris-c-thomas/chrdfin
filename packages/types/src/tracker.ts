import type { ISODateString, UUID } from "./common.js";
import type { RealTimeQuote } from "./market-data.js";

export interface Holding {
  readonly id: UUID;
  readonly portfolioId: UUID;
  readonly ticker: string;
  readonly shares: number;
  readonly costBasis: number;
  readonly avgCost: number;
  readonly firstBought?: ISODateString;
  readonly lastUpdated: string;
}

export interface HoldingWithQuote extends Holding {
  readonly quote?: RealTimeQuote;
  readonly marketValue: number;
  readonly unrealizedPnl: number;
  readonly unrealizedPnlPct: number;
  readonly weight: number;
}

export type TransactionType =
  | "buy"
  | "sell"
  | "dividend"
  | "split"
  | "transfer_in"
  | "transfer_out";

export interface Transaction {
  readonly id: UUID;
  readonly portfolioId: UUID;
  readonly ticker: string;
  readonly txType: TransactionType;
  readonly shares: number;
  readonly price: number;
  readonly fees: number;
  readonly total: number;
  readonly txDate: ISODateString;
  readonly notes?: string;
  readonly createdAt: string;
}

export interface TransactionInput {
  readonly portfolioId: UUID;
  readonly ticker: string;
  readonly txType: TransactionType;
  readonly shares: number;
  readonly price: number;
  readonly fees?: number;
  readonly txDate: ISODateString;
  readonly notes?: string;
}

export interface Watchlist {
  readonly id: UUID;
  readonly name: string;
  readonly tickers: readonly string[];
  readonly createdAt: string;
  readonly updatedAt: string;
}

export interface PortfolioSummary {
  readonly portfolioId: UUID;
  readonly totalValue: number;
  readonly totalCostBasis: number;
  readonly totalUnrealizedPnl: number;
  readonly totalUnrealizedPnlPct: number;
  readonly dayChange: number;
  readonly dayChangePct: number;
  readonly cashBalance: number;
  readonly holdingsCount: number;
}
