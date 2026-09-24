import { describe, expect, it } from "@jest/globals";
import { Box3, Vector3, Vector4 } from "three/webgpu";

import { isBoxInPlanes, STATIC_EVERYWHERE, toStaticCell } from "#/scene/static/static-cell";

/** An axis-aligned box of planes, normals in: from `min` to `max` in x, anything in y and z. */
function createSlab(min: number, max: number): Array<Vector4> {
  return [new Vector4(1, 0, 0, -min), new Vector4(-1, 0, 0, max)];
}

describe("static cells", () => {
  it("takes a draw's cell from its centre across the ground, sixty-four metres a cell", () => {
    expect(toStaticCell(new Box3(new Vector3(10, -5, 10), new Vector3(20, 50, 20)))).toBe("0,0");
    expect(toStaticCell(new Box3(new Vector3(-70, 0, 130), new Vector3(-60, 1, 140)))).toBe("-2,2");
    expect(toStaticCell(null)).toBe(STATIC_EVERYWHERE);
    expect(toStaticCell(new Box3())).toBe(STATIC_EVERYWHERE);
  });

  it("keeps a box any of which may be inside the planes, and leaves one wholly behind one of them", () => {
    const box: Box3 = new Box3(new Vector3(0, 0, 0), new Vector3(10, 10, 10));

    expect(isBoxInPlanes(box, createSlab(5, 20))).toBe(true);
    expect(isBoxInPlanes(box, createSlab(-5, 1))).toBe(true);
    expect(isBoxInPlanes(box, createSlab(11, 20))).toBe(false);
    expect(isBoxInPlanes(new Box3(), createSlab(-100, 100))).toBe(false);
  });
});
