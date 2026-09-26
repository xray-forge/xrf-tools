import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera, Vector3 } from "three/webgpu";

import { adoptRendererConventions } from "#/internals/camera-conventions";
import { toRendererFrameSize } from "#/sampling/renderer-frame-size";
import { FsrUniforms, toShadingChangeMipSide } from "#/uniforms/fsr-uniforms";

describe("FsrUniforms", () => {
  it("turns a reversed device depth back into the distance along the view, as `GetViewSpaceDepth` reads it", () => {
    const camera: PerspectiveCamera = new PerspectiveCamera(67.5, 16 / 9, 0.2, 5000);
    const uniforms: FsrUniforms = new FsrUniforms();

    adoptRendererConventions(camera);
    camera.updateProjectionMatrix();
    uniforms.take(camera, toRendererFrameSize(1920, 1080, 1.5), { offset: [0.25, -0.125], phases: 18 }, 3);

    const { x, y } = uniforms.deviceToView.value;

    for (const distance of [0.2, 1, 37.5, 5000]) {
      const depth: number = new Vector3(0, 0, -distance).applyMatrix4(camera.projectionMatrix).z;

      expect(y / (depth - x)).toBeCloseTo(distance, 3);
    }

    // The far plane stands at zero reversed, which `GetMaxDistanceInMeters` reads.
    expect(y / (0 - x)).toBeCloseTo(5000, 3);
  });

  it("turns the renderer's jitter to FSR's sense and scales the drawing to the display", () => {
    const uniforms: FsrUniforms = new FsrUniforms();

    uniforms.take(
      new PerspectiveCamera(),
      toRendererFrameSize(1920, 1080, 1.5),
      { offset: [0.25, -0.125], phases: 18 },
      3
    );

    expect(uniforms.jitter.value.toArray()).toEqual([-0.25, 0.125]);
    expect(uniforms.renderSize.value.toArray()).toEqual([1280, 720]);
    expect(uniforms.downscale.value.x).toBeCloseTo(2 / 3, 10);
    expect(uniforms.lumaMipSize.value.toArray()).toEqual([40, 22]);
    expect(uniforms.jitterPhaseCount.value).toBe(18);
    expect(uniforms.frameIndex.value).toBe(3);
    expect(toShadingChangeMipSide(20)).toBe(1);
  });
});
