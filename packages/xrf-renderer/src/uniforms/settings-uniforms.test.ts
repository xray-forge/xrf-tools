import { describe, expect, it } from "@jest/globals";

import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { ERendererRenderScale } from "#/contract/renderer-render-scale";
import { IRendererSettings } from "#/contract/renderer-settings";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";
import { SettingsUniforms, toTextureBias } from "#/uniforms/settings-uniforms";

const SETTINGS: IRendererSettings = {
  backdrop: null,
  debugView: ERendererDebugView.FINAL,
  features: RENDERER_PRESETS[ERendererPreset.BASE],
  hemiStrength: 0.5,
  isBumped: true,
  isGpuTimed: false,
  isLit: true,
  isSkyDrawn: true,
  isSkyHazed: false,
  isTextured: true,
  isWallmarkDrawn: true,
  isWireframe: false,
  pacing: DEFAULT_RENDER_FRAME_PACING,
  tonemapScale: 2,
};

describe("SettingsUniforms", () => {
  it("reads the switches and the numbers the settings give", () => {
    const uniforms: SettingsUniforms = new SettingsUniforms();

    uniforms.apply(SETTINGS);

    expect(uniforms.lit.value).toBe(1);
    expect(uniforms.textured.value).toBe(1);
    expect(uniforms.bumped.value).toBe(1);
    expect(uniforms.hemiStrength.value).toBe(0.5);
    expect(uniforms.tonemapScale.value).toBe(2);
  });

  // The textures toggle is a view's: one uniform, so no surface is put again and no texture fetched again for it.
  it("turns the textures off and on again by its uniform alone", () => {
    const uniforms: SettingsUniforms = new SettingsUniforms();

    uniforms.apply({ ...SETTINGS, isBumped: false, isLit: false, isTextured: false });

    expect(uniforms.textured.value).toBe(0);
    expect(uniforms.lit.value).toBe(0);
    expect(uniforms.bumped.value).toBe(0);

    uniforms.apply(SETTINGS);

    expect(uniforms.textured.value).toBe(1);
  });

  // A cut-out is cut against a moving threshold only where a resolve averages frames, which turns the specks of a far
  // canopy into its share of the pixel.
  it("cuts stochastically under a temporal resolve alone, and moves the frame on", () => {
    const uniforms: SettingsUniforms = new SettingsUniforms();
    const features = SETTINGS.features;

    uniforms.apply({ ...SETTINGS, features: { ...features, antialiasing: ERendererAntialiasing.TAA } });

    expect(uniforms.stochastic.value).toBe(1);

    uniforms.apply({ ...SETTINGS, features: { ...features, antialiasing: ERendererAntialiasing.SMAA } });

    expect(uniforms.stochastic.value).toBe(0);

    uniforms.advance();
    uniforms.advance();

    expect(uniforms.frame.value).toBe(2);
  });

  // The guide's extra level sharpened the distance of a frame drawn at the output's own size.
  it("samples finer under FSR 2 only while it upscales", () => {
    const fsr2 = { ...SETTINGS.features, antialiasing: ERendererAntialiasing.FSR2 };

    expect(toTextureBias({ ...fsr2, upscaling: { ...fsr2.upscaling, scale: ERendererRenderScale.NATIVE } })).toBe(0);
    expect(toTextureBias({ ...fsr2, upscaling: { ...fsr2.upscaling, scale: ERendererRenderScale.PERFORMANCE } })).toBe(
      -2
    );
  });
});
