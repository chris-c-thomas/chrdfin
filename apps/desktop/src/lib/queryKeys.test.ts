import { describe, expect, it } from "vitest";

import { qk } from "./queryKeys.js";

describe("query-key factory", () => {
  it("returns stable root keys for invalidation", () => {
    expect(qk.syncStatus()).toEqual(["sync", "status"]);
    expect(qk.prices()).toEqual(["prices"]);
    expect(qk.macro()).toEqual(["macro"]);
    expect(qk.asset()).toEqual(["asset"]);
    expect(qk.search()).toEqual(["search"]);
  });

  it("specializes prices and macro by ticker/series and date range", () => {
    const range = { start: "2025-01-01", end: "2025-12-31" };
    expect(qk.prices("SPY", range)).toEqual(["prices", "SPY", range]);
    expect(qk.prices("SPY")).toEqual(["prices", "SPY"]);
    expect(qk.macro("treasury_10_y", range)).toEqual(["macro", "treasury_10_y", range]);
  });

  it("ticker keys differ by ticker", () => {
    expect(qk.asset("SPY")).not.toEqual(qk.asset("QQQ"));
    expect(qk.search("apple")).not.toEqual(qk.search("microsoft"));
  });

  it("range keys differ when range changes", () => {
    const range1 = { start: "2025-01-01", end: "2025-06-30" };
    const range2 = { start: "2025-07-01", end: "2025-12-31" };
    expect(qk.prices("SPY", range1)).not.toEqual(qk.prices("SPY", range2));
  });
});
