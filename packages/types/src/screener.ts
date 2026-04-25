import type { AssetType } from "./market-data.js";

export interface ScreenerRangeFilter {
  readonly min?: number;
  readonly max?: number;
}

export interface ScreenerFilter {
  readonly assetType?: AssetType | "all";
  readonly sectors?: readonly string[];
  readonly marketCap?: ScreenerRangeFilter;
  readonly dividendYield?: ScreenerRangeFilter;
  readonly peRatio?: ScreenerRangeFilter;
  readonly beta?: ScreenerRangeFilter;
  readonly priceChange1d?: ScreenerRangeFilter;
  readonly priceChangeYtd?: ScreenerRangeFilter;
}

export type ScreenerSortField =
  | "ticker"
  | "marketCap"
  | "price"
  | "dayChange"
  | "ytd"
  | "yield"
  | "pe"
  | "volume";

export interface ScreenerConfig {
  readonly filter: ScreenerFilter;
  readonly sort: ScreenerSortField;
  readonly direction: "asc" | "desc";
  readonly limit: number;
}

export interface ScreenerRow {
  readonly ticker: string;
  readonly name: string;
  readonly sector?: string;
  readonly marketCap?: number;
  readonly price: number;
  readonly dayChangePct: number;
  readonly ytdPct?: number;
  readonly dividendYield?: number;
  readonly peRatio?: number;
  readonly avgVolume?: number;
  readonly priceHistory30d?: readonly number[];
}

export interface ScreenerResult {
  readonly config: ScreenerConfig;
  readonly rows: readonly ScreenerRow[];
  readonly totalMatches: number;
  readonly computedAt: string;
}
