import { describe, expect, it } from "vitest";

import { BacktestSearchSchema, ScreenerSearchSchema } from "./search-params.js";

describe("BacktestSearchSchema", () => {
  it("parses comma-separated tickers", () => {
    const result = BacktestSearchSchema.parse({ tickers: "SPY,AGG" });
    expect(result.tickers).toEqual(["SPY", "AGG"]);
  });

  it("parses comma-separated weights to numbers", () => {
    const result = BacktestSearchSchema.parse({ weights: "0.6,0.4" });
    expect(result.weights).toEqual([0.6, 0.4]);
  });

  it("applies default rebalance frequency", () => {
    const result = BacktestSearchSchema.parse({});
    expect(result.rebalance).toBe("annually");
  });

  it("rejects lowercase ticker formats", () => {
    expect(() => BacktestSearchSchema.parse({ tickers: "spy,agg" })).toThrow();
  });

  it("rejects malformed dates", () => {
    expect(() => BacktestSearchSchema.parse({ start: "2020/01/01" })).toThrow();
  });
});

describe("ScreenerSearchSchema", () => {
  it("rejects negative yieldMin", () => {
    expect(() => ScreenerSearchSchema.parse({ yieldMin: "-1" })).toThrow();
  });

  it("defaults sort direction to desc", () => {
    expect(ScreenerSearchSchema.parse({}).dir).toBe("desc");
  });

  it("defaults asset type to stocks", () => {
    expect(ScreenerSearchSchema.parse({}).asset).toBe("stocks");
  });
});
