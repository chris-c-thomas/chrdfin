/**
 * Feature flags for chrdfin domains.
 *
 * Phase 0 enables the domains that have placeholder routes; flags switch to
 * `true` as each domain ships. Untoggling a flag removes the sidebar nav item
 * (per `docs/ui-component-recipes.md` section 12) and triggers section-level
 * redirects (per `docs/route-conventions.md` section 3).
 *
 * The trio `paperTrading` / `liveTrading` / `botTrading` is reserved for the
 * post-1.0 trading roadmap (see `docs/technical-blueprint.md` § Trading
 * Module). They stay `false` until the main application is stable.
 */
export const FEATURES = {
  backtest: true,
  monteCarlo: true,
  tracker: true,
  optimizer: true,
  allocationOptimizer: true,
  calculators: true,
  marketData: true,
  news: true,
  research: false,
  reference: true,
  paperTrading: false,
  liveTrading: false,
  botTrading: false,
} as const satisfies Record<string, boolean>;

export type FeatureId = keyof typeof FEATURES;

/** Alias for routing-side imports — both names refer to the same object. */
export const featureFlags = FEATURES;

/** Compatibility alias for `import type { FeatureFlag } from "@chrdfin/config"`. */
export type FeatureFlag = FeatureId;

export function isFeatureEnabled(id: FeatureId): boolean {
  return FEATURES[id] ?? false;
}
