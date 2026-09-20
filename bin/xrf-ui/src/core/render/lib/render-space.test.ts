import { describe, expect, it } from "@jest/globals";

import { toRendererSpace, toXraySpace } from "@/core/render/lib/render-space";

describe("render space", () => {
  // The packer negates `z` on the way out, so anything read back for a person has to negate it again or it disagrees
  // with the level's own data - a spawn point, a config coordinate - by a sign on one axis only.
  it("negates the axis the packer negated, and leaves the other two", () => {
    expect(toXraySpace({ x: -243.75, y: 12.5, z: 87.25 })).toEqual({ x: -243.75, y: 12.5, z: -87.25 });
    expect(toRendererSpace({ x: -243.75, y: 12.5, z: -87.25 })).toEqual({ x: -243.75, y: 12.5, z: 87.25 });
  });

  it("round trips", () => {
    const point = { x: 1.5, y: -2.25, z: 3.75 };

    expect(toRendererSpace(toXraySpace(point))).toEqual(point);
  });

  it("leaves the origin where it is, which is the one place both spaces agree on", () => {
    expect(toXraySpace({ x: 0, y: 0, z: 0 })).toEqual({ x: 0, y: 0, z: 0 });
  });
});
