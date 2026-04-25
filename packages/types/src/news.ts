import type { ISODateString, ISODateTimeString, UUID } from "./common.js";

export interface NewsArticle {
  readonly id: UUID;
  readonly source: string;
  readonly externalId?: string;
  readonly title: string;
  readonly description?: string;
  readonly url: string;
  readonly publishedAt: ISODateTimeString;
  readonly tickers?: readonly string[];
  readonly tags?: readonly string[];
  readonly isBookmarked: boolean;
}

export interface EarningsCalendarEntry {
  readonly ticker: string;
  readonly reportDate: ISODateString;
  readonly fiscalQuarter?: string;
  readonly estimateEps?: number;
  readonly actualEps?: number;
  readonly estimateRev?: number;
  readonly actualRev?: number;
}

export interface RSSFeedSource {
  readonly id: string;
  readonly name: string;
  readonly url: string;
  readonly category: string;
}

export interface NewsFilter {
  readonly tickers?: readonly string[];
  readonly source?: string;
  readonly category?: string;
  readonly query?: string;
  readonly bookmarkedOnly?: boolean;
  readonly limit?: number;
}
