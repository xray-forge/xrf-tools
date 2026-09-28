import { describe, expect, it } from "@jest/globals";
import { Box3, Sphere, Vector3, Vector4 } from "three/webgpu";

import { isBoxInPlanes, isSphereInPlanes, toSphereVisibility } from "#/visibility/plane-tests";
import { EVisibility } from "#/visibility/visibility";

/** An axis-aligned box of planes, normals in: from `min` to `max` in x, anything in y and z. */
function createSlab(min: number, max: number): Array<Vector4> {
  return [new Vector4(1, 0, 0, -min), new Vector4(-1, 0, 0, max)];
}

describe("plane tests", () => {
  it("keeps a box any of which may be inside the planes, and leaves one wholly behind one of them", () => {
    const box: Box3 = new Box3(new Vector3(0, 0, 0), new Vector3(10, 10, 10));

    expect(isBoxInPlanes(box, createSlab(5, 20))).toBe(true);
    expect(isBoxInPlanes(box, createSlab(-5, 1))).toBe(true);
    expect(isBoxInPlanes(box, createSlab(11, 20))).toBe(false);
    expect(isBoxInPlanes(new Box3(), createSlab(-100, 100))).toBe(false);
  });

  it("keeps a sphere reaching over a plane, and leaves one wholly behind it", () => {
    expect(isSphereInPlanes(new Sphere(new Vector3(-1, 0, 0), 2), createSlab(0, 10))).toBe(true);
    expect(isSphereInPlanes(new Sphere(new Vector3(-3, 0, 0), 2), createSlab(0, 10))).toBe(false);
  });

  it("tells a sphere wholly inside the planes, one across one of them, and one wholly behind one apart", () => {
    const slab: Array<Vector4> = createSlab(0, 10);

    expect(toSphereVisibility(5, 0, 0, 2, slab)).toBe(EVisibility.INSIDE);
    expect(toSphereVisibility(9, 0, 0, 2, slab)).toBe(EVisibility.INTERSECTS);
    expect(toSphereVisibility(13, 0, 0, 2, slab)).toBe(EVisibility.OUTSIDE);
  });
});
