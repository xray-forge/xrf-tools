import { describe, expect, it } from "@jest/globals";

import { toRenderVector } from "@/core/render/lib/scene/render-vector";

describe("toRenderVector", () => {
  it("reads each component, a missing one as zero", () => {
    expect(toRenderVector({ x: 1, y: null, z: -3 })).toEqual([1, 0, -3]);
  });
});
