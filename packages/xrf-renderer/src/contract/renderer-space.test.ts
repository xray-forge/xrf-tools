import { describe, expect, it } from "@jest/globals";

import { toEngineVector, toRendererVector } from "#/contract/renderer-space";

describe("toRendererVector", () => {
  it("negates the engine's z alone", () => {
    expect(toRendererVector([1, 2, 3])).toEqual([1, 2, -3]);
  });

  it("leaves a zero z as zero, never negative zero", () => {
    expect(Object.is(toRendererVector([1, 2, 0])[2], 0)).toBe(true);
  });
});

describe("toEngineVector", () => {
  it("takes a vector back as the engine states it", () => {
    expect(toEngineVector(toRendererVector([4, -5, 6]))).toEqual([4, -5, 6]);
  });
});
