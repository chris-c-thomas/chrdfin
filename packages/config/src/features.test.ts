import { describe, expect, it } from "vitest";

import { FEATURES, isFeatureEnabled } from "./features.js";

describe("isFeatureEnabled", () => {
  it("returns true for enabled flags", () => {
    expect(isFeatureEnabled("backtest")).toBe(true);
    expect(isFeatureEnabled("tracker")).toBe(true);
    expect(isFeatureEnabled("optimizer")).toBe(true);
    expect(isFeatureEnabled("allocationOptimizer")).toBe(true);
    expect(isFeatureEnabled("reference")).toBe(true);
  });

  it("returns false for deferred / future flags", () => {
    expect(isFeatureEnabled("research")).toBe(false);
    expect(isFeatureEnabled("paperTrading")).toBe(false);
    expect(isFeatureEnabled("liveTrading")).toBe(false);
    expect(isFeatureEnabled("botTrading")).toBe(false);
  });

  it("FEATURES is a complete map of known domains", () => {
    const expected = [
      "backtest",
      "monteCarlo",
      "tracker",
      "optimizer",
      "allocationOptimizer",
      "calculators",
      "marketData",
      "news",
      "research",
      "reference",
      "paperTrading",
      "liveTrading",
      "botTrading",
    ];
    expect(Object.keys(FEATURES).sort()).toEqual(expected.sort());
  });
});
