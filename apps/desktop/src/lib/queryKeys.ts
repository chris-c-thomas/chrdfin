/**
 * Central query-key factory for every TanStack Query call in the app.
 *
 * Centralizing keys avoids divergence between the hooks that read data
 * and the mutation/invalidation paths that need to invalidate them.
 * After a sync completes, `useSyncDataMutation` calls
 * `invalidateQueries({ queryKey: qk.prices() })` and so on — the partial
 * keys returned by the no-arg overloads make broad invalidation cheap.
 */

interface DateRange {
  readonly start: string;
  readonly end: string;
}

export const qk = {
  syncStatus: () => ["sync", "status"] as const,

  prices: (ticker?: string, range?: DateRange) => {
    if (!ticker) return ["prices"] as const;
    if (!range) return ["prices", ticker] as const;
    return ["prices", ticker, range] as const;
  },

  asset: (ticker?: string) => (ticker ? (["asset", ticker] as const) : (["asset"] as const)),

  search: (query?: string) => (query ? (["search", query] as const) : (["search"] as const)),

  macro: (series?: string, range?: DateRange) => {
    if (!series) return ["macro"] as const;
    if (!range) return ["macro", series] as const;
    return ["macro", series, range] as const;
  },
} as const;
