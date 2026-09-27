import { describe, expect, it } from "@jest/globals";
import { DEFAULT_RENDER_FRAME_PACING, DEFAULT_RENDERER_FEATURE_CHOICE, resolveRendererFeatures } from "@xrf/renderer";

import { DEFAULT_LEVEL_FEATURE_OPTIONS } from "@/core/level/lib/features/level-feature-options";
import { DEFAULT_LEVEL_LIGHTING } from "@/core/level/lib/lighting/level-lighting";
import { DEFAULT_LEVEL_LOD_OPTIONS } from "@/core/level/lib/lod/level-lod-options";
import { DEFAULT_LEVEL_RENDER_CONFIG } from "@/core/level/lib/render/level-render-config";
import { ILevelRendererSettingsInputs, toLevelRendererSettings } from "@/core/level/lib/render/level-render-view";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";

function toInputs(isTimed: boolean, isAdvancedStatsVisible: boolean): ILevelRendererSettingsInputs {
  return {
    config: DEFAULT_LEVEL_RENDER_CONFIG,
    features: { ...resolveRendererFeatures(DEFAULT_RENDERER_FEATURE_CHOICE), isGpuTimed: isTimed },
    lighting: DEFAULT_LEVEL_LIGHTING,
    lod: DEFAULT_LEVEL_LOD_OPTIONS,
    options: { ...DEFAULT_LEVEL_VIEW_OPTIONS, isAdvancedStatsVisible },
    pacing: DEFAULT_RENDER_FRAME_PACING,
    view: DEFAULT_LEVEL_FEATURE_OPTIONS,
  };
}

describe("toLevelRendererSettings", () => {
  it("times the GPU only while the readout shows what each pass cost", () => {
    expect(toLevelRendererSettings(toInputs(true, false)).features.isGpuTimed).toBe(false);
    expect(toLevelRendererSettings(toInputs(true, true)).features.isGpuTimed).toBe(true);
  });

  it("leaves timing off where the settings turn it off, whatever the readout shows", () => {
    expect(toLevelRendererSettings(toInputs(false, true)).features.isGpuTimed).toBe(false);
  });
});
