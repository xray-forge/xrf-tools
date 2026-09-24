import { describe, expect, it } from "@jest/globals";

import { toPackedCoordinate } from "#/geometry/renderer-packed-coordinate";

describe("toPackedCoordinate", () => {
  it("adds a baked coordinate's low bytes from the tangent and binormal, over 1024", () => {
    const packed = {
      binormal: new Uint8Array([0, 0, 0, 0, 0, 0, 0, 51]),
      tangent: new Uint8Array([0, 0, 0, 0, 0, 0, 0, 255]),
      uv: new Int16Array([0, 0, 1024, -512]),
    };

    expect(toPackedCoordinate(packed, 2, 1)).toEqual([(1024 + 1) / 1024, (-512 + 0.2) / 1024]);
  });

  it("takes a tree's shorts alone, over 2048, whatever its tangent frame carries", () => {
    const packed = { tangent: new Uint8Array([0, 0, 0, 200]), uv: new Int16Array([2048, 1024, 7, 9]) };

    expect(toPackedCoordinate(packed, 4, 0)).toEqual([1, 0.5]);
  });
});
