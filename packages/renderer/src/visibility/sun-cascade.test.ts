import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3, Vector4, WebGPUCoordinateSystem } from "three/webgpu";

import { SunCascade } from "#/visibility/sun-cascade";

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

const DOWN: Vector3 = new Vector3(0, -1, 0);

describe("SunCascade", () => {
  it("leads the camera by a third of its width along where it looks, across the light", () => {
    const cascade: SunCascade = new SunCascade();

    cascade.fit(createCamera(0, 10, 0), DOWN, 20, 2048, 400);

    // Straight down, the map lies in the ground's plane: centred seven metres ahead, the camera well inside it.
    expect(isInside(cascade.planes, new Vector3(0, 0, 7))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, 0, -2))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, 0, 18))).toBe(false);
    expect(isInside(cascade.planes, new Vector3(11, 0, 7))).toBe(false);
    expect(cascade.texel).toBeCloseTo(20 / 2048);
  });

  it("takes casters from as far towards the sun as its reach, and receivers as far away from it", () => {
    const cascade: SunCascade = new SunCascade();

    cascade.fit(createCamera(0, 10, 0), DOWN, 20, 2048, 400);

    expect(isInside(cascade.planes, new Vector3(0, 400, 7))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, 420, 7))).toBe(false);
    // A camera flying high still has its ground: the far side lies as far below as the reach, and the margin past it.
    expect(isInside(cascade.planes, new Vector3(0, -380, 7))).toBe(true);
    expect(isInside(cascade.planes, new Vector3(0, -440, 7))).toBe(false);
  });

  // A map that slid with every centimetre the camera moves shimmers along every shadow's edge.
  it("moves a whole texel at a time, and says so only when it does", () => {
    const cascade: SunCascade = new SunCascade();

    cascade.fit(createCamera(0, 10, 0), DOWN, 20, 2048, 400);

    const version: number = cascade.version;
    const position: Vector3 = cascade.camera.position.clone();

    cascade.fit(createCamera(0.001, 10, 0), DOWN, 20, 2048, 400);

    expect(cascade.version).toBe(version);
    expect(cascade.camera.position.distanceTo(position)).toBe(0);

    cascade.fit(createCamera(0.5, 10, 0), DOWN, 20, 2048, 400);

    expect(cascade.version).toBe(version + 1);
    expect(
      Math.abs(cascade.camera.position.x / cascade.texel - Math.round(cascade.camera.position.x / cascade.texel))
    ).toBeLessThan(1e-6);
  });

  it("draws from the sun's side, looking along its light", () => {
    const cascade: SunCascade = new SunCascade();
    const light: Vector3 = new Vector3(1, -2, 0.5).normalize();

    cascade.fit(createCamera(0, 10, 0), light, 40, 1024, 400);

    expect(cascade.camera.getWorldDirection(new Vector3()).dot(light)).toBeCloseTo(1);
    expect(cascade.camera.right - cascade.camera.left).toBeCloseTo(40);
  });
});
