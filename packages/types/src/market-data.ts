import type { ISODateString, ISODateTimeString } from "./common.js";

export type AssetType = "stock" | "etf" | "mutual_fund" | "index";

export interface AssetMetadata {
  readonly ticker: string;
  readonly name: string;
  readonly assetType: AssetType;
  readonly sector?: string;
  readonly industry?: string;
  readonly exchange?: string;
  readonly marketCap?: number;
  readonly firstDate?: ISODateString;
  readonly lastDate?: ISODateString;
  readonly isActive: boolean;
  readonly metadata?: Record<string, unknown>;
}

export interface DailyPrice {
  readonly ticker: string;
  readonly date: ISODateString;
  readonly open?: number;
  readonly high?: number;
  readonly low?: number;
  readonly close: number;
  readonly adjClose: number;
  readonly volume?: number;
}

export interface DividendEvent {
  readonly ticker: string;
  readonly exDate: ISODateString;
  readonly amount: number;
  readonly divType?: "regular" | "special" | "return_of_capital";
}

export interface RealTimeQuote {
  readonly ticker: string;
  readonly price: number;
  readonly bid?: number;
  readonly ask?: number;
  readonly bidSize?: number;
  readonly askSize?: number;
  readonly dayChange: number;
  readonly dayChangePct: number;
  readonly dayHigh?: number;
  readonly dayLow?: number;
  readonly dayVolume?: number;
  readonly previousClose: number;
  readonly timestamp: ISODateTimeString;
}

export interface OptionContract {
  readonly contractSymbol: string;
  readonly underlying: string;
  readonly expiry: ISODateString;
  readonly strike: number;
  readonly type: "call" | "put";
  readonly bid?: number;
  readonly ask?: number;
  readonly last?: number;
  readonly impliedVolatility?: number;
  readonly delta?: number;
  readonly gamma?: number;
  readonly theta?: number;
  readonly vega?: number;
  readonly rho?: number;
  readonly openInterest?: number;
  readonly volume?: number;
}

export interface OptionsChain {
  readonly underlying: string;
  readonly asOf: ISODateTimeString;
  readonly expiries: readonly ISODateString[];
  readonly contracts: readonly OptionContract[];
}

export interface FundamentalData {
  readonly ticker: string;
  readonly peRatio?: number;
  readonly pegRatio?: number;
  readonly priceToBook?: number;
  readonly dividendYield?: number;
  readonly eps?: number;
  readonly beta?: number;
  readonly marketCap?: number;
  readonly enterpriseValue?: number;
  readonly profitMargin?: number;
  readonly returnOnEquity?: number;
  readonly debtToEquity?: number;
  readonly fiftyTwoWeekHigh?: number;
  readonly fiftyTwoWeekLow?: number;
}

export interface MacroSeriesPoint {
  readonly seriesId: string;
  readonly date: ISODateString;
  readonly value: number;
}
