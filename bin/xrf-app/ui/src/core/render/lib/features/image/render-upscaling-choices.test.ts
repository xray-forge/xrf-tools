import { describe, expect, it } from "@jest/globals";
import { ERenderAntialiasing, ERenderLightModel, ERenderTonemapMode } from "@xrf/ipc/types/xrf-renderer";

import { RENDER_UPSCALING_SETTINGS } from "@/core/render/lib/features/image/render-upscaling-choices";
import { isRenderSettingInert } from "@/core/render/lib/features/render-setting";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/choice/render-preset";

/** DLSS over the engine's light model and tonemap, the one way the scene holds no linear light. */
const FEATURES: IRenderFeatureSettings = {
  ...RENDER_PRESETS[ERenderPreset.BASE],
  antialiasing: ERenderAntialiasing.DLSS,
  lightModel: ERenderLightModel.ENGINE,
  tonemap: ERenderTonemapMode.ENGINE,
};

/**
 * @param features - Every feature as drawn.
 * @returns Whether ray reconstruction shows disabled with them.
 */
function isRayReconstructionInert(features: IRenderFeatureSettings): boolean {
  return isRenderSettingInert(RENDER_UPSCALING_SETTINGS.isRayReconstructed, features.upscaling, features);
}

describe("RENDER_UPSCALING_SETTINGS", () => {
  // As `RenderViewOptions::is_scene_linear` decides: the Enhanced tonemap or the physical light.
  it("offers ray reconstruction wherever the renderer reconstructs: DLSS over linear light", () => {
    expect(isRayReconstructionInert(FEATURES)).toBe(true);
    expect(isRayReconstructionInert({ ...FEATURES, tonemap: ERenderTonemapMode.ENHANCED })).toBe(false);
    expect(isRayReconstructionInert({ ...FEATURES, lightModel: ERenderLightModel.PHYSICAL })).toBe(false);
    expect(
      isRayReconstructionInert({
        ...FEATURES,
        antialiasing: ERenderAntialiasing.TAA,
        lightModel: ERenderLightModel.PHYSICAL,
      })
    ).toBe(true);
  });
});
