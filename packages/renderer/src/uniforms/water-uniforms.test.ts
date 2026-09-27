import { describe, expect, it } from "@jest/globals";

import { DEFAULT_RENDERER_WATER_SETTINGS } from "#/contract/renderer-features";
import { WaterUniforms } from "#/uniforms/water-uniforms";

describe("WaterUniforms", () => {
  it("reads the settings, the lighting's intensity, and the time", () => {
    const uniforms: WaterUniforms = new WaterUniforms();

    uniforms.apply({ ...DEFAULT_RENDERER_WATER_SETTINGS, distortion: 0.1, isSoft: false, waveHeight: 0.05 });
    uniforms.take({ waterIntensity: 0.5 });
    uniforms.update(12.5);

    expect(uniforms.distortion.value).toBe(0.1);
    expect(uniforms.soft.value).toBe(0);
    expect(uniforms.waveHeight.value).toBe(0.05);
    expect(uniforms.intensity.value).toBe(0.5);
    expect(uniforms.time.value).toBe(12.5);
  });

  // The pipelines the water compiles against bind a float depth, whatever the frame's depth behind is later.
  it("reads a far depth before the frame's is made", () => {
    expect((new WaterUniforms().depth.value.image as { data: Float32Array }).data[0]).toBeGreaterThan(1000);
  });
});
