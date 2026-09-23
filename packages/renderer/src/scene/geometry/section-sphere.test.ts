import { describe, expect, it } from "@jest/globals";
import { Sphere } from "three/webgpu";

import { toSectionSphere } from "#/scene/geometry/section-sphere";

/** Four vertices on the x axis, at 0, 2, 10 and 14. */
const POSITIONS: Float32Array = new Float32Array([0, 0, 0, 2, 0, 0, 10, 0, 0, 14, 0, 0]);

describe("toSectionSphere", () => {
  it("bounds only the vertices its range of indices reaches", () => {
    const sphere: Sphere = toSectionSphere(POSITIONS, new Uint16Array([0, 1, 0, 2, 3, 2]), 3, 3);

    expect(sphere.center.toArray()).toEqual([12, 0, 0]);
    expect(sphere.radius).toBe(2);
  });

  it("reads vertices in order for a geometry without indices", () => {
    const sphere: Sphere = toSectionSphere(POSITIONS, null, 0, 2);

    expect(sphere.center.toArray()).toEqual([1, 0, 0]);
    expect(sphere.radius).toBe(1);
  });

  it("is empty for a range reaching nothing", () => {
    expect(toSectionSphere(POSITIONS, null, 0, 0).isEmpty()).toBe(true);
  });
});
