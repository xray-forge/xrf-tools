import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3, WebGPUCoordinateSystem } from "three/webgpu";

import { ISunViewRay } from "#/visibility/sun-view-ray";
import { SunViewRays } from "#/visibility/sun-view-rays";

describe("SunViewRays", () => {
  it("starts the view's four edges at the corners of the near plane, running out through the far one's", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera(90, 2, 0.5, 100);
    const rays: SunViewRays = new SunViewRays();

    camera.coordinateSystem = WebGPUCoordinateSystem;
    camera.position.set(1, 2, 3);
    camera.updateMatrixWorld(true);
    rays.reset(camera);

    // Ninety degrees high and twice as wide: a corner half a metre ahead stands half a metre up and a metre across.
    const corners: Array<Vector3> = rays.rays.map((ray: ISunViewRay) => ray.origin.clone().sub(camera.position));

    [
      [-1, -0.5, -0.5],
      [1, -0.5, -0.5],
      [-1, 0.5, -0.5],
      [1, 0.5, -0.5],
    ].forEach(([x, y, z]: Array<number>, index: number) =>
      expect(corners[index].distanceTo(new Vector3(x, y, z))).toBeCloseTo(0)
    );
    rays.rays.forEach((ray: ISunViewRay, index: number) => {
      expect(ray.direction.clone().normalize().dot(corners[index].clone().normalize())).toBeCloseTo(1);
      expect(rays.near[index].origin).toEqual(ray.origin);
      expect(rays.near[index].direction).toEqual(ray.direction);
    });
  });
});
