import { describe, expect, it } from "@jest/globals";

import { ABSENT_VALUE, formatNumber, formatPercent } from "@/lib/format/number";

describe("formatNumber", () => {
  it("renders a fixed number of decimals", () => {
    expect(formatNumber(1.23456, 3)).toBe("1.235");
    expect(formatNumber(-0.5, 2)).toBe("-0.50");
    expect(formatNumber(0, 0)).toBe("0");
  });

  it("marks an absent value rather than showing a zero", () => {
    // A non-finite rust float serialises to null, so this is the ipc contract rather than a defensive check.
    expect(formatNumber(null, 3)).toBe(ABSENT_VALUE);
  });

  it("marks a non-finite value, which no fixed rendering can express", () => {
    expect(formatNumber(Number.NaN, 3)).toBe(ABSENT_VALUE);
    expect(formatNumber(Number.POSITIVE_INFINITY, 3)).toBe(ABSENT_VALUE);
    expect(formatNumber(Number.NEGATIVE_INFINITY, 3)).toBe(ABSENT_VALUE);
  });

  it("accepts a caller's own placeholder for every absent case", () => {
    expect(formatNumber(null, 3, "unknown")).toBe("unknown");
    expect(formatNumber(Number.NaN, 3, "unknown")).toBe("unknown");
    expect(formatNumber(Number.POSITIVE_INFINITY, 3, "unknown")).toBe("unknown");
  });

  it("ignores the placeholder when there is a number to render", () => {
    expect(formatNumber(1, 1, "unknown")).toBe("1.0");
    expect(formatNumber(0, 0, "unknown")).toBe("0");
  });

  it("accepts an empty placeholder, for a surface that wants the value simply gone", () => {
    // Distinct from the default: a table cell may prefer blank to a dash, and `""` must not fall back to it.
    expect(formatNumber(null, 2, "")).toBe("");
  });

  it("defaults to the shared placeholder when a caller supplies none", () => {
    expect(formatNumber(null, 2)).toBe(ABSENT_VALUE);
  });

  it("rounds rather than truncating, and keeps negative zero readable", () => {
    expect(formatNumber(1.005, 2)).toBe("1.00");
    expect(formatNumber(1.006, 2)).toBe("1.01");
    expect(formatNumber(-0.0001, 2)).toBe("-0.00");
  });
});

describe("formatPercent", () => {
  it("takes the fraction and writes the percentage", () => {
    expect(formatPercent(0.65)).toBe("65%");
    expect(formatPercent(1)).toBe("100%");
    expect(formatPercent(0)).toBe("0%");
  });

  it("keeps the decimals it is asked for", () => {
    expect(formatPercent(0.1234, 1)).toBe("12.3%");
  });
});
