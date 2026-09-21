import { describe, expect, it } from "@jest/globals";

import { toMean, toWorst } from "@/lib/math/series";

describe("toMean", () => {
  it("means a series", () => {
    expect(toMean([2, 4, 6])).toBe(4);
  });

  // A reading of nothing is not a reading of anything else, and every caller reports it as "not measured yet".
  it("reports zero for a series of nothing", () => {
    expect(toMean([])).toBe(0);
  });

  it("keeps the fraction rather than rounding it away", () => {
    expect(toMean([1, 2])).toBe(1.5);
  });
});

describe("toWorst", () => {
  it("takes the largest of a series", () => {
    expect(toWorst([2, 9, 4])).toBe(9);
  });

  it("reports zero for a series of nothing", () => {
    expect(toWorst([])).toBe(0);
  });

  // Costs, all of them, so nothing below zero is a worse one.
  it("never reports below zero", () => {
    expect(toWorst([-5, -1])).toBe(0);
  });
});
