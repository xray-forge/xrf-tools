import { describe, expect, it } from "@jest/globals";
import { Matrix4, PerspectiveCamera, Sphere, Vector3, WebGPUCoordinateSystem } from "three/webgpu";

import { CullView } from "#/visibility/cull-view";
import { collectVisibleInstances, toInstanceSpheres } from "#/visibility/instance-spheres";

/** Sixteen floats placing an instance at a point, scaled evenly. */
function place(x: number, y: number, z: number, scale: number = 1): Array<number> {
  return new Matrix4().makeScale(scale, scale, scale).setPosition(x, y, z).toArray();
}

describe("toInstanceSpheres", () => {
  it("stands the mesh's sphere in every place, scaled with it", () => {
    const spheres: Float32Array = toInstanceSpheres(
      new Sphere(new Vector3(0, 1, 0), 2),
      new Float32Array([...place(10, 0, 0), ...place(0, 0, 5, 3)]),
      new Matrix4()
    );

    expect(Array.from(spheres)).toEqual([10, 1, 0, 2, 0, 3, 5, 6]);
  });

  it("places every instance by the object's own matrix as well", () => {
    const spheres: Float32Array = toInstanceSpheres(
      new Sphere(new Vector3(), 1),
      new Float32Array(place(1, 0, 0)),
      new Matrix4().makeTranslation(0, 0, -4)
    );

    expect(Array.from(spheres)).toEqual([1, 0, -4, 1]);
  });
});

describe("collectVisibleInstances", () => {
  it("collects the places the view sees, in their order", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 1, 100);

    camera.coordinateSystem = WebGPUCoordinateSystem;
    camera.updateProjectionMatrix();
    camera.updateMatrixWorld();

    const view: CullView = new CullView();
    const into: Uint32Array = new Uint32Array(4);

    view.take(camera);

    // In front, behind, past the far plane, in front again.
    const count: number = collectVisibleInstances(
      view,
      new Float32Array([0, 0, -10, 1, 0, 0, 10, 1, 0, 0, -500, 1, 2, 0, -20, 1]),
      into
    );

    expect(count).toBe(2);
    expect(Array.from(into.subarray(0, count))).toEqual([0, 3]);
  });
});
