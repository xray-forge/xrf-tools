import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3, Vector4, WebGPUCoordinateSystem } from "three/webgpu";

import { SunCascade } from "#/visibility/sun-cascade";
import { SunViewRays } from "#/visibility/sun-view-rays";

/** A camera at the given point looking along `+z`. */
function createCamera(x: number, y: number, z: number): PerspectiveCamera {
  const camera: PerspectiveCamera = new PerspectiveCamera(67.5, 1.7, 0.2, 5000);

  camera.coordinateSystem = WebGPUCoordinateSystem;
  camera.position.set(x, y, z);
  camera.lookAt(x, y, z + 1);
  camera.updateMatrixWorld(true);

  return camera;
}

/** Whether a point is inside every plane. */
function isInside(planes: ReadonlyArray<Vector4>, point: Vector3): boolean {
  return planes.every((plane: Vector4) => plane.x * point.x + plane.y * point.y + plane.z * point.z + plane.w >= -1e-6);
}

/** The engine's three cascades, placed one after another along the camera's view. */
function fitChain(camera: PerspectiveCamera, light: Vector3): Array<SunCascade> {
  const rays: SunViewRays = new SunViewRays();

  rays.reset(camera);

  return [20, 40, 160].map((width: number) => {
    const cascade: SunCascade = new SunCascade();

    cascade.fit(camera, rays, light, width, 2048, 400);

    return cascade;
  });
}

/** The first cascade alone, from the camera's near plane. */
function fitFirst(camera: PerspectiveCamera, light: Vector3, width: number = 20): SunCascade {
  const rays: SunViewRays = new SunViewRays();
  const cascade: SunCascade = new SunCascade();

  rays.reset(camera);
  cascade.fit(camera, rays, light, width, 2048, 400);

  return cascade;
}

const DOWN: Vector3 = new Vector3(0, -1, 0);

describe("SunCascade", () => {
  // A square centred on the camera spends half of itself behind it, on what is never seen.
  it("brings its back edge up to the camera's near plane, and covers what is ahead", () => {
    const cascade: SunCascade = fitFirst(createCamera(0, 1.7, 0), DOWN);

    expect(isInside(cascade.planes, new Vector3(0, 0, 1))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, 0, 19))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, 0, -1))).toBe(false);
    expect(isInside(cascade.planes, new Vector3(0, 0, 21))).toBe(false);
    expect(isInside(cascade.planes, new Vector3(9, 0, 10))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(11, 0, 10))).toBe(false);
    expect(cascade.texel).toBeCloseTo(20 / 2048);
  });

  it("starts each cascade where the view's edges leave the one before it", () => {
    const [near, middle, far] = fitChain(createCamera(0, 1.7, 0), DOWN);

    // Each square reaches on from where the one before it leaves the view, as far as holding the first quarter of its
    // own width of the view lets it.
    expect(isInside(near.planes, new Vector3(0, 0, 19))).toBe(true);
    expect(isInside(middle.planes, new Vector3(0, 0, 38))).toBe(true);
    expect(isInside(middle.planes, new Vector3(0, 0, 41))).toBe(false);
    expect(isInside(far.planes, new Vector3(0, 0, 150))).toBe(true);
    expect(isInside(far.planes, new Vector3(0, 0, 160))).toBe(false);
  });

  // Looking down with the sun ahead, an edge runs back out through the back of every square the engine brings up to
  // where it starts, and the ground nearest the camera was in none of them.
  it("holds the ground nearest the camera in every cascade, whatever the placement says", () => {
    const camera: PerspectiveCamera = createCamera(0, 3.8, 0);
    const light: Vector3 = new Vector3(0, -0.5, -0.866).normalize();

    camera.lookAt(0, 3.8 - Math.tan((Math.PI / 180) * 50), 1);
    camera.updateMatrixWorld(true);

    const cascades: Array<SunCascade> = fitChain(camera, light);
    const rays: SunViewRays = new SunViewRays();

    rays.reset(camera);

    for (const ray of rays.near) {
      // Where each corner of the view meets the ground, when it does in front of the camera.
      const t: number = -ray.origin.y / ray.direction.y;

      if (t > 0 && t < 10) {
        const ground: Vector3 = ray.origin.clone().addScaledVector(ray.direction, t);

        expect(isInside(cascades[0].planes, ground)).toBe(true);
      }
    }
  });

  it("takes casters from as far towards the sun as its reach, and receivers as far away from it", () => {
    const cascade: SunCascade = fitFirst(createCamera(0, 1.7, 0), DOWN);

    expect(isInside(cascade.planes, new Vector3(0, 400, 10))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, 420, 10))).toBe(false);
    // A camera flying high still has its ground: the far side lies as far below as the reach, and the margin past it.
    expect(isInside(cascade.planes, new Vector3(0, -380, 10))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, -440, 10))).toBe(false);
  });

  // A map that slid with every centimetre the camera moves shimmers along every shadow's edge.
  it("moves a whole texel at a time, and says so only when it does", () => {
    const cascade: SunCascade = new SunCascade();
    const rays: SunViewRays = new SunViewRays();

    function fit(x: number): void {
      const camera: PerspectiveCamera = createCamera(x, 1.7, 0);

      rays.reset(camera);
      cascade.fit(camera, rays, DOWN, 20, 2048, 400);
    }

    fit(0);

    const version: number = cascade.version;
    const position: Vector3 = cascade.camera.position.clone();

    fit(0.001);

    expect(cascade.version).toBe(version);
    expect(cascade.camera.position.distanceTo(position)).toBe(0);

    fit(0.5);

    expect(cascade.version).toBe(version + 1);
    expect(
      Math.abs(cascade.camera.position.x / cascade.texel - Math.round(cascade.camera.position.x / cascade.texel))
    ).toBeLessThan(1e-6);
  });

  it("stays over the camera looking along the light", () => {
    const camera: PerspectiveCamera = createCamera(0, 10, 0);

    camera.lookAt(0, 0, 0);
    camera.updateMatrixWorld(true);

    const cascade: SunCascade = fitFirst(camera, DOWN);

    expect(isInside(cascade.planes, new Vector3(9, 0, 9))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(-9, 0, -9))).toBe(true);
  });

  it("draws from the sun's side, looking along its light", () => {
    const light: Vector3 = new Vector3(1, -2, 0.5).normalize();
    const cascade: SunCascade = fitFirst(createCamera(0, 10, 0), light, 40);

    expect(cascade.camera.getWorldDirection(new Vector3()).dot(light)).toBeCloseTo(1);
    expect(cascade.camera.right - cascade.camera.left).toBeCloseTo(40);
  });
});
