import { describe, expect, it } from "@jest/globals";

import { toPickTexel } from "#/scene/object/pick-texel";
import { EPickKind } from "#/shader/pick-kind";

describe("toPickTexel", () => {
  it("reads which draw and place a texel names, rounding what crossed as floats", () => {
    expect(toPickTexel(new Float32Array([1, 4096.0001, 12.9999, 2.5]))).toEqual({
      distance: 2.5,
      draw: 4096,
      kind: EPickKind.STATIC,
      place: 13,
    });
    expect(toPickTexel([2, 17, 0, 40])?.kind).toBe(EPickKind.PLAIN);
  });

  it("reads nothing where the target stood as it was cleared, or holds a kind no pick writes", () => {
    expect(toPickTexel(new Float32Array(4))).toBeNull();
    expect(toPickTexel([3, 1, 1, 1])).toBeNull();
  });
});
