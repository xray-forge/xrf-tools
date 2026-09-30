import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector4 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { CullView } from "#/visibility/cull-view";
import { EVisibility } from "#/visibility/visibility";

/** A camera at the origin looking down -z, seeing from one metre to a hundred, in the renderer's reversed depth. */
function createCamera(): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(90, 1, 1, 100);

  adoptRendererConventions(camera);
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

  it("culls what lies past the far plane and short of the near one, under reversed depth", () => {
    const view: CullView = new CullView();

    view.take(createCamera());

    expect(view.classify(0, 0, -150, 10)).toBe(EVisibility.OUTSIDE);
    expect(view.classify(0, 0, -0.5, 0.1)).toBe(EVisibility.OUTSIDE);
  });

  it("finds the far plane where it replaces it, facing back along the view, under reversed depth", () => {
    const camera: PerspectiveCamera = createCamera();
    const view: CullView = new CullView();

    view.take(camera);

    const far: Vector4 = view.planes[4];

    expect(camera.reversedDepth).toBe(true);
    expect(far.z).toBeCloseTo(1);
    expect(far.w).toBeCloseTo(100);
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

  it("moves its version when how far the view sees changes, the camera staying put", () => {
    const camera: PerspectiveCamera = createCamera();
    const view: CullView = new CullView();

    view.take(camera, 40);

    const first: number = view.version;

    view.take(camera, 40);

    expect(view.version).toBe(first);

    view.take(camera, 60);

    expect(view.version).toBe(first + 1);
    expect(view.classify(0, 0, -50, 5)).toBe(EVisibility.INSIDE);
  });

  // `CalcSSA` against `r_ssaDISCARD`: a radius over its squared distance, dropped at the threshold or below it.
  it("drops a sphere too small on screen, from where the camera stands, only while a threshold is taken", () => {
    const camera: PerspectiveCamera = createCamera();
    const view: CullView = new CullView();

    view.take(camera, Infinity, 0.001);

    expect(view.isDiscarded(0, 0, -10, 0.2)).toBe(false);
    expect(view.isDiscarded(0, 0, -10, 0.05)).toBe(true);

    const first: number = view.version;

    view.take(camera);

    expect(view.version).toBe(first + 1);
    expect(view.isDiscarded(0, 0, -10, 0.05)).toBe(false);
  });
});
