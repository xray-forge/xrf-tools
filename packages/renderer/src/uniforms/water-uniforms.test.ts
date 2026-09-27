import { describe, expect, it } from "@jest/globals";

import { DEFAULT_RENDERER_WATER_SETTINGS } from "#/contract/renderer-features";
import { DEFAULT_RENDERER_LIGHTING } from "#/lighting/default-lighting";
import { WaterUniforms } from "#/uniforms/water-uniforms";

describe("WaterUniforms", () => {
  it("reads the settings, the lighting's skies and intensity, and the time", () => {
    const uniforms: WaterUniforms = new WaterUniforms();

    uniforms.apply({ ...DEFAULT_RENDERER_WATER_SETTINGS, distortion: 0.1, isSoft: false, waveHeight: 0.05 });
    uniforms.take({ ...DEFAULT_RENDERER_LIGHTING, sky: { blend: 0.25, textures: ["a", "b"] }, waterIntensity: 0.5 });
    uniforms.update(12.5);

    expect(uniforms.distortion.value).toBe(0.1);
    expect(uniforms.soft.value).toBe(0);
    expect(uniforms.waveHeight.value).toBe(0.05);
    expect(uniforms.skyBlend.value).toBe(0.25);
    expect(uniforms.intensity.value).toBe(0.5);
    expect(uniforms.time.value).toBe(12.5);
  });

  // The pipelines the water compiles against bind cubes and a float depth, whatever the lighting names later.
  it("samples a sky cube and a far depth before anything is put", () => {
    const uniforms: WaterUniforms = new WaterUniforms();

    expect(uniforms.skies.every((sky) => (sky.value as { isCubeTexture?: boolean }).isCubeTexture)).toBe(true);
    expect((uniforms.depth.value.image as { data: Float32Array }).data[0]).toBeGreaterThan(1000);
  });
});
