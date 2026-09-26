import { describe, expect, it } from "@jest/globals";
import { Matrix4, PerspectiveCamera, Vector3 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import {
  jitterProjection,
  TemporalJitter,
  toHalton,
  toTemporalJitter,
  toTemporalJitterPhases,
} from "#/pass/temporal-jitter";
import { toRendererFrameSize } from "#/sampling/renderer-frame-size";
import { MotionUniforms } from "#/uniforms/motion-uniforms";

describe("temporal jitter", () => {
  it("jitters by Halton (2, 3), every offset within half a pixel and their mean near the pixel's centre", () => {
    expect([1, 2, 3, 4].map((index: number) => toHalton(index, 2))).toEqual([0.5, 0.25, 0.75, 0.125]);
    expect(toHalton(1, 3)).toBeCloseTo(1 / 3, 10);

    const offsets: Array<readonly [number, number]> = Array.from(
      { length: toTemporalJitterPhases(1920, 1920) },
      (_, phase: number) => toTemporalJitter(phase)
    );

    for (const [x, y] of offsets) {
      expect(Math.abs(x)).toBeLessThan(0.5);
      expect(Math.abs(y)).toBeLessThan(0.5);
    }

    const mean: Array<number> = [0, 1].map(
      (axis: number) => offsets.reduce((sum: number, offset) => sum + offset[axis], 0) / offsets.length
    );

    expect(Math.abs(mean[0])).toBeLessThan(0.1);
    expect(Math.abs(mean[1])).toBeLessThan(0.1);
  });

  it("cycles through as many places as FSR 2 does for each scale, from the widths drawn and shown", () => {
    const widths: Array<number> = [1, 1.5, 1.7, 2].map(
      (upscale: number) => toRendererFrameSize(3280, 1930, upscale).renderWidth
    );

    expect(widths).toEqual([3280, 2187, 1929, 1640]);
    expect(widths.map((width: number) => toTemporalJitterPhases(3280, width))).toEqual([8, 17, 23, 32]);
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

  it("jitters a camera of its own, leaving the view's alone, and moves through its cycle from the start", () => {
    const motion: MotionUniforms = new MotionUniforms();
    const jitter: TemporalJitter = new TemporalJitter(motion);
    const view: PerspectiveCamera = new PerspectiveCamera(60, 2, 0.2, 1000);

    adoptRendererConventions(view);
    view.position.set(1, 2, 3);
    view.updateMatrixWorld();
    jitter.resize(toRendererFrameSize(800, 400, 2));

    const projection: Matrix4 = view.projectionMatrix.clone();
    const drawn: PerspectiveCamera = jitter.take(view);
    const [x, y] = toTemporalJitter(0);

    expect(drawn).not.toBe(view);
    expect(view.projectionMatrix.equals(projection)).toBe(true);
    expect(drawn.matrixWorld.equals(view.matrixWorld)).toBe(true);
    expect(drawn.projectionMatrix.elements[8] - projection.elements[8]).toBeCloseTo((2 * x) / 400, 10);
    expect(motion.jitter.value.toArray()).toEqual([x, y]);
    expect(jitter.state.phases).toBe(32);

    // Taken twice in one frame, it jitters once.
    jitter.take(view);
    expect(drawn.projectionMatrix.elements[8] - projection.elements[8]).toBeCloseTo((2 * x) / 400, 10);

    jitter.advance();
    expect(jitter.state.offset).toEqual(toTemporalJitter(1));

    jitter.resize(toRendererFrameSize(800, 400, 1));
    expect(jitter.state).toEqual({ offset: toTemporalJitter(0), phases: 8 });

    jitter.dispose();
    expect(motion.jitter.value.toArray()).toEqual([0, 0]);
  });
});
