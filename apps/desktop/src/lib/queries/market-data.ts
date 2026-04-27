import { useQuery } from "@tanstack/react-query";
import { invoke } from "@tauri-apps/api/core";

import type {
  AssetMetadata,
  DailyPrice,
  MacroObservation,
  MacroSeriesId,
  TickerSearchResponse,
} from "@chrdfin/types";

import { qk } from "@/lib/queryKeys.js";

const PRICES_STALE_MS = 5 * 60 * 1000;
const MACRO_STALE_MS = 60 * 60 * 1000;
const ASSET_STALE_MS = 60 * 60 * 1000;
const SEARCH_STALE_MS = 30 * 1000;

interface DateRange {
  readonly start: string;
  readonly end: string;
}

/**
 * EOD bars for `ticker` between `start` and `end` (inclusive). The
 * underlying Tauri command transparently fills any missing window via
 * the orchestrator before reading from DuckDB.
 */
export function usePrices(ticker: string | undefined, range: DateRange | undefined) {
  return useQuery<readonly DailyPrice[], Error>({
    queryKey: qk.prices(ticker, range),
    queryFn: async () =>
      invoke<readonly DailyPrice[]>("get_prices", {
        input: { ticker, start: range?.start, end: range?.end },
      }),
    enabled: Boolean(ticker && range?.start && range?.end),
    staleTime: PRICES_STALE_MS,
  });
}

/** One macroeconomic series within `[start, end]`. Local-only read. */
export function useMacroSeries(series: MacroSeriesId | undefined, range: DateRange | undefined) {
  return useQuery<readonly MacroObservation[], Error>({
    queryKey: qk.macro(series, range),
    queryFn: async () =>
      invoke<readonly MacroObservation[]>("get_macro_series", {
        input: { seriesId: series, start: range?.start, end: range?.end },
      }),
    enabled: Boolean(series && range?.start && range?.end),
    staleTime: MACRO_STALE_MS,
  });
}

/** Asset metadata, on-demand fetched if not yet in DuckDB. */
export function useAssetMetadata(ticker: string | undefined) {
  return useQuery<AssetMetadata, Error>({
    queryKey: qk.asset(ticker),
    queryFn: async () => invoke<AssetMetadata>("get_asset_metadata", { input: { ticker } }),
    enabled: Boolean(ticker),
    staleTime: ASSET_STALE_MS,
  });
}

/**
 * Hybrid local+remote ticker search. Returns the merged hit list; the
 * remote leg's failures are swallowed inside the Rust command, so the
 * hook only sees the merged success path.
 */
export function useTickerSearch(query: string, limit?: number) {
  const trimmed = query.trim();
  return useQuery<TickerSearchResponse, Error>({
    queryKey: qk.search(trimmed),
    queryFn: async () =>
      invoke<TickerSearchResponse>("search_tickers", {
        input: { query: trimmed, limit },
      }),
    enabled: trimmed.length > 0,
    staleTime: SEARCH_STALE_MS,
  });
}
