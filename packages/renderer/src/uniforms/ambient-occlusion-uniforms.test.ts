import { describe, expect, it } from "@jest/globals";
import { PerspectiveCamera } from "three/webgpu";

import { DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS } from "#/contract/renderer-features";
import { AMBIENT_OCCLUSION_FINAL_POWER, AmbientOcclusionUniforms } from "#/uniforms/ambient-occlusion-uniforms";

describe("AmbientOcclusionUniforms", () => {
  // A metre at a metre spans the target's width over the view's width there: two tangents of half the field.
  it("sizes a metre in the search target's pixels, and scales XeGTAO's curve by the strength", () => {
    const uniforms: AmbientOcclusionUniforms = new AmbientOcclusionUniforms();
    const camera: PerspectiveCamera = new PerspectiveCamera(90, 2, 0.2, 1000);

    camera.updateProjectionMatrix();
    uniforms.take({ ...DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS, radius: 2, strength: 0.5 }, camera, 800, 400);

    expect(uniforms.radius.value).toBe(2);
    expect(uniforms.power.value).toBeCloseTo(AMBIENT_OCCLUSION_FINAL_POWER * 0.5, 8);
    // Half-width in view space at a metre is tan(45°) times the aspect: two metres over 800 pixels.
    expect(uniforms.spread.value).toBeCloseTo((2 * 2) / 800, 8);
    expect(uniforms.reach.value).toBe(100);
  });
});
