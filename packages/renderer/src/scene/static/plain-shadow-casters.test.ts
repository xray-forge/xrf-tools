import { describe, expect, it } from "@jest/globals";
import { BufferGeometry, Mesh, Sphere, Vector3, Vector4 } from "three/webgpu";

import { PlainShadowCasters } from "#/scene/static/plain-shadow-casters";

/** A box two metres either way of the origin, as six planes pointing in. */
const PLANES: ReadonlyArray<Vector4> = [
  new Vector4(1, 0, 0, 2),
  new Vector4(-1, 0, 0, 2),
  new Vector4(0, 1, 0, 2),
  new Vector4(0, -1, 0, 2),
  new Vector4(0, 0, 1, 2),
  new Vector4(0, 0, -1, 2),
];

describe("PlainShadowCasters", () => {
  it("shows a view only the twins reaching into it that draw anything, and says whether any does", () => {
    const casters: PlainShadowCasters = new PlainShadowCasters();
    const [inside, reaching, outside] = [
      new Mesh(new BufferGeometry()),
      new Mesh(new BufferGeometry()),
      new Mesh(new BufferGeometry()),
    ];
    const reach: Sphere = new Sphere(new Vector3(3, 0, 0), 1.5);

    casters.put(inside, new Sphere(new Vector3(), 1));
    casters.put(reaching, reach);
    casters.put(outside, new Sphere(new Vector3(10, 0, 0), 1));

    expect(casters.show(PLANES)).toBe(true);
    expect([inside.visible, reaching.visible, outside.visible]).toEqual([true, true, false]);

    // The part moved away, its sphere with it: the caller's own sphere is what is tested.
    reach.center.set(5, 0, 0);
    casters.setDrawing(inside, false);
    casters.show(PLANES);

    expect([inside.visible, reaching.visible]).toEqual([false, false]);

    casters.release(inside);
    casters.release(reaching);

    expect(casters.show(PLANES)).toBe(false);
    expect(casters.scene.children).toEqual([outside]);
  });
});
