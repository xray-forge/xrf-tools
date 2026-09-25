import { describe, expect, it } from "@jest/globals";
import { Matrix4, PerspectiveCamera, Vector3 } from "three/webgpu";

import { jitterProjection, TEMPORAL_JITTER, toHalton } from "#/pass/temporal-antialias-pass";

describe("temporal antialiasing", () => {
  it("jitters by Halton (2, 3), every offset within half a pixel and their mean near the pixel's centre", () => {
    expect([1, 2, 3, 4].map((index: number) => toHalton(index, 2))).toEqual([0.5, 0.25, 0.75, 0.125]);
    expect(toHalton(1, 3)).toBeCloseTo(1 / 3, 10);

    for (const [x, y] of TEMPORAL_JITTER) {
      expect(Math.abs(x)).toBeLessThan(0.5);
      expect(Math.abs(y)).toBeLessThan(0.5);
    }

    const mean: Array<number> = [0, 1].map(
      (axis: number) => TEMPORAL_JITTER.reduce((sum, offset) => sum + offset[axis], 0) / TEMPORAL_JITTER.length
    );

    expect(Math.abs(mean[0])).toBeLessThan(0.1);
    expect(Math.abs(mean[1])).toBeLessThan(0.1);
  });

  // The texel at `m` shows the scene at `m + 0.5 + jitter`: a point moves the other way on the screen, `y` down.
  it("moves every point on the screen by the jitter the other way, whatever its depth", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera(60, 2, 0.2, 1000);
    const width: number = 800;
    const height: number = 400;

    camera.updateProjectionMatrix();

    const jittered: Matrix4 = camera.projectionMatrix.clone();

    jitterProjection(jittered, 0.25, -0.375, width, height);

    for (const point of [new Vector3(1, 2, -5), new Vector3(-30, 4, -300)]) {
      const plain: Vector3 = point.clone().applyMatrix4(camera.projectionMatrix);
      const shifted: Vector3 = point.clone().applyMatrix4(jittered);

      // Pixels across and down from clip: x half the width per unit, y half the height the other way.
      expect(((shifted.x - plain.x) * width) / 2).toBeCloseTo(-0.25, 6);
      expect((-(shifted.y - plain.y) * height) / 2).toBeCloseTo(0.375, 6);
      expect(shifted.z).toBeCloseTo(plain.z, 10);
    }
  });
});
