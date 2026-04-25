import { describe, expect, it } from "vitest";

import {
  formatAbbreviated,
  formatCurrency,
  formatDelta,
  formatNumber,
  formatPercent,
} from "./format.js";

describe("formatCurrency", () => {
  it("formats positives with thousands separators", () => {
    expect(formatCurrency(847293.14)).toBe("$847,293.14");
  });

  it("uses leading minus on negatives by default", () => {
    expect(formatCurrency(-1234.56)).toBe("-$1,234.56");
  });

  it("wraps negatives in parens in accounting mode", () => {
    expect(formatCurrency(-1234.56, { accounting: true })).toBe("($1,234.56)");
  });

  it("prepends + when signed", () => {
    expect(formatCurrency(2841.07, { signed: true })).toBe("+$2,841.07");
  });

  it("returns placeholder for null/undefined/NaN", () => {
    expect(formatCurrency(null)).toBe("—");
    expect(formatCurrency(undefined)).toBe("—");
    expect(formatCurrency(NaN)).toBe("—");
  });
});

describe("formatPercent", () => {
  it("treats input as already-scaled by default", () => {
    expect(formatPercent(34.7)).toBe("34.70%");
  });

  it("scales decimal input when scale=decimal", () => {
    expect(formatPercent(0.347, { scale: "decimal" })).toBe("34.70%");
  });

  it("applies signed prefix on positives", () => {
    expect(formatPercent(34.7, { signed: true })).toBe("+34.70%");
  });
});

describe("formatNumber", () => {
  it("formats with thousands separators", () => {
    expect(formatNumber(12345.678)).toBe("12,345.68");
  });
});

describe("formatAbbreviated", () => {
  it("uses T for trillions", () => {
    expect(formatAbbreviated(2_400_000_000_000)).toBe("2.40T");
  });

  it("supports a prefix", () => {
    expect(formatAbbreviated(12_400_000, { prefix: "$" })).toBe("$12.40M");
  });

  it("preserves sign on negatives", () => {
    expect(formatAbbreviated(-1_500_000)).toBe("-1.50M");
  });
});

describe("formatDelta", () => {
  it("composes absolute and percent change", () => {
    expect(formatDelta(2841.07, 0.34)).toBe("+$2,841.07 / +0.34%");
  });
});
