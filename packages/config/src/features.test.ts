import { describe, expect, it } from "vitest";

import { FEATURES, isFeatureEnabled } from "./features.js";

describe("isFeatureEnabled", () => {
  it("returns true for enabled flags", () => {
    expect(isFeatureEnabled("backtest")).toBe(true);
    expect(isFeatureEnabled("tracker")).toBe(true);
  });

  it("returns false for disabled flags", () => {
    expect(isFeatureEnabled("optimizer")).toBe(false);
    expect(isFeatureEnabled("research")).toBe(false);
  });

  it("FEATURES is a complete map of known domains", () => {
    const expected = [
      "backtest",
      "monteCarlo",
      "tracker",
      "optimizer",
      "calculators",
      "marketData",
      "news",
      "research",
    ];
    expect(Object.keys(FEATURES).sort()).toEqual(expected.sort());
  });
});
