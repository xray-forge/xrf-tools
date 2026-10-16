import { describe, expect, it } from "@jest/globals";

import { addVectors, normalise, scaleVector } from "#/vector";

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

describe("normalise", () => {
  it("keeps the direction at length one", () => {
    expect(normalise([0, 3, 4])).toEqual([0, 0.6, 0.8]);
  });

  it("leaves a zero vector as it is", () => {
    expect(normalise([0, 0, 0])).toEqual([0, 0, 0]);
  });
});
