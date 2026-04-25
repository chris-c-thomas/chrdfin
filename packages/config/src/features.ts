/**
 * Feature flags for chrdfin domains.
 *
 * Phase 0 enables the domains that have placeholder routes; flags switch to
 * `true` as each domain ships. Untoggling a flag removes the sidebar nav item
 * (per `docs/ui-component-recipes.md` section 12) and triggers section-level
 * redirects (per `docs/route-conventions.md` section 3).
 */
export const FEATURES = {
  backtest: true,
  monteCarlo: true,
  tracker: true,
  optimizer: false,
  calculators: true,
  marketData: true,
  news: true,
  research: false,
} as const satisfies Record<string, boolean>;

export type FeatureId = keyof typeof FEATURES;

/** Alias for routing-side imports — both names refer to the same object. */
export const featureFlags = FEATURES;

/** Compatibility alias for `import type { FeatureFlag } from "@chrdfin/config"`. */
export type FeatureFlag = FeatureId;

export function isFeatureEnabled(id: FeatureId): boolean {
  return FEATURES[id] ?? false;
}
