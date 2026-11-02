import { describe, expect, it } from "@jest/globals";

import { toRenderGridStep } from "./render-grid-step";

describe("toRenderGridStep", () => {
  // A grid is read, not just looked at: a cell has to be a number somebody can add up and compare against a
  // coordinate, which `extent / cells` almost never is.
  it("rounds to one, two or five times a power of ten", () => {
    expect(toRenderGridStep(1000, 24)).toBe(50);
    expect(toRenderGridStep(1000, 40)).toBe(50);
    expect(toRenderGridStep(1000, 100)).toBe(10);
    expect(toRenderGridStep(10, 40)).toBe(0.5);
    expect(toRenderGridStep(0.4, 25)).toBe(0.02);
  });

  it("takes the first round step at least as coarse as the one asked for", () => {
    // Never finer: a step below the request would draw more lines than the caller budgeted for.
    expect(toRenderGridStep(210, 100)).toBe(5);
    expect(toRenderGridStep(190, 100)).toBe(2);
  });

  it("answers a usable step for an extent that is not one", () => {
    expect(toRenderGridStep(0, 25)).toBe(1);
    expect(toRenderGridStep(Number.NaN, 25)).toBe(1);
    expect(toRenderGridStep(-1000, 20)).toBe(50);
    expect(toRenderGridStep(100, 0)).toBe(100);
  });
});
