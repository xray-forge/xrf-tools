import { describe, expect, it } from "@jest/globals";

import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { IRendererSettings } from "#/contract/renderer-settings";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";
import { SettingsUniforms } from "#/uniforms/settings-uniforms";

const SETTINGS: IRendererSettings = {
  backdrop: null,
  debugView: ERendererDebugView.FINAL,
  features: RENDERER_PRESETS[ERendererPreset.BASE],
  hemiStrength: 0.5,
  isBumped: true,
  isGpuTimed: false,
  isLit: true,
  isSkyDrawn: true,
  isTextured: true,
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
});
