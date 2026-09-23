import { describe, expect, it } from "@jest/globals";
import { InterleavedBufferAttribute, Matrix4, PerspectiveCamera, WebGPUCoordinateSystem } from "three/webgpu";

import { SceneGeometry } from "#/scene/geometry/scene-geometry";
import { SceneInstances } from "#/scene/object/scene-instances";
import { CullView } from "#/visibility/cull-view";

/** A unit triangle around its origin. */
function createGeometry(): SceneGeometry {
  return new SceneGeometry({ groups: [], position: new Float32Array([-0.5, 0, 0, 0.5, 0, 0, 0, 0.5, 0]) });
}

/** A view from the origin down -z, to a hundred metres. */
function createView(x: number = 0): CullView {
  const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 1, 100);
  const view: CullView = new CullView();

  camera.coordinateSystem = WebGPUCoordinateSystem;
  camera.position.x = x;
  camera.updateProjectionMatrix();
  camera.updateMatrixWorld();
  view.take(camera);

  return view;
}

/** Sixteen floats placing an instance at a point. */
function place(x: number, y: number, z: number): Array<number> {
  return new Matrix4().makeTranslation(x, y, z).toArray();
}

describe("SceneInstances", () => {
  // In front, behind the camera, in front again.
  const transforms: Float32Array = new Float32Array([...place(0, 0, -10), ...place(0, 0, 10), ...place(1, 0, -20)]);

  it("draws the places a view sees, copied to the front in their order", () => {
    const instances: SceneInstances = new SceneInstances(createGeometry(), { transforms });

    expect(instances.cull(createView())).toBe(2);
    expect(instances.geometry.instanceCount).toBe(2);

    const drawn = (instances.geometry.getAttribute("instanceMatrix0") as InterleavedBufferAttribute).data.array;

    expect(Array.from(drawn.slice(0, 32))).toEqual([...place(0, 0, -10), ...place(1, 0, -20)]);
  });

  it("uploads nothing when a moved view still sees the same places", () => {
    const instances: SceneInstances = new SceneInstances(createGeometry(), { transforms });
    const columns = (instances.geometry.getAttribute("instanceMatrix0") as InterleavedBufferAttribute).data;

    instances.cull(createView(0));

    const version: number = columns.version;

    expect(instances.cull(createView(0.1))).toBe(2);
    expect(columns.version).toBe(version);
  });

  it("carries each drawn place's hemisphere terms with it", () => {
    const instances: SceneInstances = new SceneInstances(createGeometry(), {
      hemi: new Float32Array([1, 0.1, 2, 0.2, 3, 0.3]),
      transforms,
    });

    instances.cull(createView());

    expect(Array.from(instances.geometry.getAttribute("instanceHemi").array.slice(0, 4))).toEqual([
      1,
      expect.closeTo(0.1),
      3,
      expect.closeTo(0.3),
    ]);
  });
});
