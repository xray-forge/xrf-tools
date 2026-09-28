import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3, WebGPUCoordinateSystem } from "three/webgpu";

import { SunCascadeBasis } from "#/visibility/sun-cascade-basis";
import { placeSunCascade } from "#/visibility/sun-cascade-placement";
import { ISunViewRay } from "#/visibility/sun-view-ray";
import { SunViewRays } from "#/visibility/sun-view-rays";

function createCamera(at: Vector3, target: Vector3): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(67.5, 1.7, 0.2, 5000);

  camera.coordinateSystem = WebGPUCoordinateSystem;
  camera.position.copy(at);
  camera.lookAt(target);
  camera.updateMatrixWorld(true);

  return camera;
}

function place(camera: PerspectiveCamera, light: Vector3, width: number): { center: Vector3; rays: SunViewRays } {
  const basis: SunCascadeBasis = new SunCascadeBasis();
  const rays: SunViewRays = new SunViewRays();
  const center: Vector3 = camera.getWorldPosition(new Vector3());

  basis.take(light);
  rays.reset(camera);
  placeSunCascade(center, camera.getWorldDirection(new Vector3()), rays, basis, width);

  return { center, rays };
}

const DOWN: Vector3 = new Vector3(0, -1, 0);

describe("placeSunCascade", () => {
  // The near plane is 0.2 ahead: the back side comes up to it and is held 3% of a width short, the sampling's edge.
  it("brings the side the view looks away from up to where the view starts, short by the held margin", () => {
    const { center } = place(createCamera(new Vector3(0, 1.5, 0), new Vector3(0, 1.5, 1)), DOWN, 20);

    expect(center.x).toBeCloseTo(0);
    expect(center.y).toBeCloseTo(1.5);
    expect(center.z).toBeCloseTo(0.2 + 20 * 0.47);
  });

  it("carries every edge on to where it leaves the square, ahead of where it started", () => {
    const camera: PerspectiveCamera = createCamera(new Vector3(0, 1.5, 0), new Vector3(0, 1.5, 1));
    const { center, rays } = place(camera, DOWN, 20);

    rays.rays.forEach((ray: ISunViewRay, index: number) => {
      const start: ISunViewRay = rays.near[index];
      const reach: number = Math.max(Math.abs(ray.origin.x - center.x), Math.abs(ray.origin.z - center.z));

      expect(ray.origin.distanceTo(start.origin)).toBeGreaterThan(0);
      expect(reach).toBeCloseTo(10);
    });
  });

  it("stays over the camera looking along the light, and carries no edge on", () => {
    const camera: PerspectiveCamera = createCamera(new Vector3(3, 10, 4), new Vector3(3, 0, 4));
    const { center, rays } = place(camera, DOWN, 20);

    expect(center.x).toBeCloseTo(3);
    expect(center.z).toBeCloseTo(4);
    rays.rays.forEach((ray: ISunViewRay, index: number) => expect(ray.origin).toEqual(rays.near[index].origin));
  });
});
