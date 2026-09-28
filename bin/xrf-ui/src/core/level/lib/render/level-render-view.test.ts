import { describe, expect, it } from "@jest/globals";
import { DEFAULT_RENDER_FRAME_PACING, DEFAULT_RENDERER_FEATURE_CHOICE, resolveRendererFeatures } from "@xrf/renderer";

import { DEFAULT_LEVEL_LIGHTING } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_LOD_OPTIONS } from "@/core/level/lib/lod/level-lod-options";
import { DEFAULT_LEVEL_RENDER_CONFIG } from "@/core/level/lib/render/level-render-config";
import { ILevelRendererSettingsInputs, toLevelRendererSettings } from "@/core/level/lib/render/level-render-view";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";

function toInputs(isGpuTimed: boolean): ILevelRendererSettingsInputs {
  return {
    config: DEFAULT_LEVEL_RENDER_CONFIG,
    features: { ...resolveRendererFeatures(DEFAULT_RENDERER_FEATURE_CHOICE), isGpuTimed },
    lighting: DEFAULT_LEVEL_LIGHTING,
    lod: DEFAULT_LEVEL_LOD_OPTIONS,
    options: DEFAULT_LEVEL_VIEW_OPTIONS,
    pacing: DEFAULT_RENDER_FRAME_PACING,
    view: mockLevelFeatureOptions(),
  };
}

describe("toLevelRendererSettings", () => {
  it("times the GPU exactly as the settings do, whatever the toolbar shows", () => {
    expect(toLevelRendererSettings(toInputs(true)).features.isGpuTimed).toBe(true);
    expect(toLevelRendererSettings(toInputs(false)).features.isGpuTimed).toBe(false);
    expect(
      toLevelRendererSettings({ ...toInputs(true), options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isStatsVisible: false } })
        .features.isGpuTimed
    ).toBe(true);
  });
});
