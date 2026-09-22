import { describe, expect, it } from "@jest/globals";

import { toDegrees, toRadians } from "#/angle";

describe("toDegrees", () => {
  it("turns the unit the maths uses into the one a person reads", () => {
    expect(toDegrees(Math.PI)).toBeCloseTo(180);
    expect(toDegrees(Math.PI / 2)).toBeCloseTo(90);
    expect(toDegrees(0)).toBe(0);
  });
});

describe("toRadians", () => {
  it("turns the unit a control offers into the one the maths takes", () => {
    expect(toRadians(180)).toBeCloseTo(Math.PI);
    expect(toRadians(-90)).toBeCloseTo(-Math.PI / 2);
  });

  it("is the inverse of the other", () => {
    expect(toDegrees(toRadians(67.5))).toBeCloseTo(67.5);
  });
});
