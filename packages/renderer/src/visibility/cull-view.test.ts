import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, WebGPUCoordinateSystem } from "three/webgpu";

import { CullView } from "#/visibility/cull-view";
import { EVisibility } from "#/visibility/visibility";

/** A camera at the origin looking down -z, seeing from one metre to a hundred. */
function createCamera(): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 1, 100);

  camera.coordinateSystem = WebGPUCoordinateSystem;
  camera.updateProjectionMatrix();
  camera.updateMatrixWorld();

  return camera;
}

describe("CullView", () => {
  it("tells a sphere wholly inside, one across a plane, and one outside apart", () => {
    const view: CullView = new CullView();

    view.take(createCamera());

    expect(view.classify(0, 0, -50, 1)).toBe(EVisibility.INSIDE);
    expect(view.classify(0, 0, -100, 5)).toBe(EVisibility.INTERSECTS);
    expect(view.classify(0, 0, 10, 1)).toBe(EVisibility.OUTSIDE);
  });

  it("culls what lies past the far plane", () => {
    const view: CullView = new CullView();

    view.take(createCamera());

    expect(view.classify(0, 0, -150, 10)).toBe(EVisibility.OUTSIDE);
  });

  it("brings its far plane in to how far the view sees, leaving the camera's own", () => {
    const camera: PerspectiveCamera = createCamera();
    const view: CullView = new CullView();

    view.take(camera, 40);

    expect(view.classify(0, 0, -30, 1)).toBe(EVisibility.INSIDE);
    expect(view.classify(0, 0, -50, 5)).toBe(EVisibility.OUTSIDE);
    expect(camera.far).toBe(100);
  });

  it("keeps its version while the camera stays put, and moves it when the camera moves", () => {
    const camera: PerspectiveCamera = createCamera();
    const view: CullView = new CullView();

    view.take(camera);

    const first: number = view.version;

    view.take(camera);

    expect(view.version).toBe(first);

    camera.position.x = 1;
    camera.updateMatrixWorld();
    view.take(camera);

    expect(view.version).toBe(first + 1);
  });
});
