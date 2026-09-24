import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera } from "three/webgpu";

import { DEFAULT_RENDERER_LOD_SETTINGS } from "#/contract/renderer-features";
import { LodUniforms } from "#/uniforms/lod-uniforms";

describe("LodUniforms", () => {
  it("takes the engine's thresholds over the drawing's pixels, scaled by a field of view against ninety", () => {
    const uniforms: LodUniforms = new LodUniforms();
    const camera: PerspectiveCamera = new PerspectiveCamera(90);

    camera.position.set(1, 2, 3);
    camera.updateMatrixWorld();
    uniforms.take({ ...DEFAULT_RENDERER_LOD_SETTINGS, geometryLod: 1 }, 1000, 1000, camera);

    expect(uniforms.lodA.value).toBeCloseTo((64 / 3) ** 2 / 1e6, 9);
    expect(uniforms.lodB.value).toBeCloseTo((48 / 3) ** 2 / 1e6, 9);
    expect(uniforms.discard.value).toBeCloseTo(3.5 ** 2 / 1e6, 9);
    expect(uniforms.camera.value.toArray()).toEqual([1, 2, 3]);

    // Half the field of view spends four times the pixels on a clump: every threshold quarters.
    camera.fov = 45;
    uniforms.take({ ...DEFAULT_RENDERER_LOD_SETTINGS, geometryLod: 1 }, 1000, 1000, camera);

    expect(uniforms.lodA.value).toBeCloseTo((64 / 3) ** 2 / 4e6, 9);
  });

  it("says whether anything changed, and turns impostors off as a flag the cull reads", () => {
    const uniforms: LodUniforms = new LodUniforms();
    const camera: PerspectiveCamera = new PerspectiveCamera(90);

    expect(uniforms.take(DEFAULT_RENDERER_LOD_SETTINGS, 100, 100, camera)).toBe(true);
    expect(uniforms.take(DEFAULT_RENDERER_LOD_SETTINGS, 100, 100, camera)).toBe(false);
    expect(uniforms.take({ ...DEFAULT_RENDERER_LOD_SETTINGS, isImpostors: false }, 100, 100, camera)).toBe(true);
    expect(uniforms.isEnabled.value).toBe(0);
  });
});
