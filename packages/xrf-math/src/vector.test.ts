import { describe, expect, it } from "@jest/globals";

import { addVectors, crossProduct, dotProduct, normalise, scaleVector } from "#/vector";

describe("addVectors", () => {
  it("adds each component", () => {
    expect(addVectors([1, 2, 3], [10, 20, 30])).toEqual([11, 22, 33]);
  });
});

describe("scaleVector", () => {
  it("scales each component", () => {
    expect(scaleVector([1, -2, 3], -2)).toEqual([-2, 4, -6]);
  });
});

describe("dotProduct", () => {
  it("is zero across perpendicular vectors and the squared length along one", () => {
    expect(dotProduct([1, 0, 0], [0, 1, 0])).toBe(0);
    expect(dotProduct([1, 2, 2], [1, 2, 2])).toBe(9);
  });
});

describe("crossProduct", () => {
  it("turns x into z through y, and the other way round backwards", () => {
    expect(crossProduct([1, 0, 0], [0, 1, 0])).toEqual([0, 0, 1]);
    expect(crossProduct([0, 1, 0], [1, 0, 0])).toEqual([0, 0, -1]);
  });
});

describe("normalise", () => {
  it("keeps the direction at length one", () => {
    expect(normalise([0, 3, 4])).toEqual([0, 0.6, 0.8]);
  });

  it("leaves a zero vector as it is", () => {
    expect(normalise([0, 0, 0])).toEqual([0, 0, 0]);
  });
});
