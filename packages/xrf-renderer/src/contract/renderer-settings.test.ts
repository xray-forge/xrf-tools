import { describe, expect, it } from "@jest/globals";

import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { IRendererFeatureSettings } from "#/contract/renderer-feature-settings";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { IRendererSettings, toRendererSettings } from "#/contract/renderer-settings";
import { RENDERER_MAX_SHADOW_CASCADES } from "#/contract/renderer-shadow-settings";
import { DEFAULT_RENDER_FRAME_PACING } from "#/frame/render-frame-pacing";

const BASE: IRendererFeatureSettings = RENDERER_PRESETS[ERendererPreset.BASE];

const SENT: IRendererSettings = {
  backdrop: 0x202020,
  debugView: ERendererDebugView.NORMAL,
  features: BASE,
  hemiStrength: 0.5,
  isBumped: true,
  isGpuTimed: false,
  isLit: true,
  isSkyDrawn: false,
  isSkyHazed: false,
  isTextured: true,
  isWallmarkDrawn: true,
  isWireframe: false,
  pacing: DEFAULT_RENDER_FRAME_PACING,
  tonemapScale: 2,
};

/** The cascades the renderer takes of a run of that many sent. */
function toTakenCascades(count: number): ReadonlyArray<number> {
  const cascades: Array<number> = Array.from({ length: count }, (_: unknown, index: number) => (index + 1) * 10);

  return toRendererSettings({ ...SENT, features: { ...BASE, shadows: { ...BASE.shadows, cascades } } }).features.shadows
    .cascades;
}

describe("toRendererSettings", () => {
  it("takes the settings as sent, their features held to the schema", () => {
    const sent: IRendererSettings = { ...SENT, features: { ...BASE, grass: { ...BASE.grass, height: 0 } } };

    expect(toRendererSettings(sent)).toEqual({ ...sent, features: { ...BASE, grass: { ...BASE.grass, height: 0.5 } } });
  });

  it("takes the view's switches as sent, textures off among them", () => {
    expect(toRendererSettings({ ...SENT, isBumped: false, isTextured: false })).toMatchObject({
      isBumped: false,
      isTextured: false,
    });
  });

  it("takes a run of cascades up to the most the sun samples, and the preset's run in place of a longer one", () => {
    expect(toTakenCascades(RENDERER_MAX_SHADOW_CASCADES)).toHaveLength(RENDERER_MAX_SHADOW_CASCADES);
    expect(toTakenCascades(RENDERER_MAX_SHADOW_CASCADES + 1)).toEqual(BASE.shadows.cascades);
  });
});
