import { describe, expect, it } from "@jest/globals";

import { toPoint3 } from "#/point";

describe("toPoint3", () => {
  it("keeps the axes the wire carries", () => {
    expect(toPoint3({ x: 1.5, y: -2, z: 3 })).toEqual({ x: 1.5, y: -2, z: 3 });
  });

  it("reads an absent axis as zero, which is what a non-finite float arrives as", () => {
    expect(toPoint3({ x: null, y: 4, z: null })).toEqual({ x: 0, y: 4, z: 0 });
  });

  it("reads an absent point as the origin", () => {
    expect(toPoint3(null)).toEqual({ x: 0, y: 0, z: 0 });
    expect(toPoint3(undefined)).toEqual({ x: 0, y: 0, z: 0 });
  });
});
