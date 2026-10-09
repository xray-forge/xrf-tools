import { describe, expect, it } from "@jest/globals";

import { toSphereFitDistance } from "#/sphere-fit";

describe("toSphereFitDistance", () => {
  it("stands back until the sphere touches the edges of the view", () => {
    // At ninety degrees the half angle is forty five, so the distance is the radius over its sine.
    expect(toSphereFitDistance(1, Math.PI / 2, 1)).toBeCloseTo(Math.SQRT2);
  });

  it("scales with the margin left around the sphere", () => {
    expect(toSphereFitDistance(2, Math.PI / 3, 1.5)).toBeCloseTo(6);
  });
});
