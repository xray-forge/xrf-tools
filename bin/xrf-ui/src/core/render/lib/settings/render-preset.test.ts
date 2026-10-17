import { describe, expect, it } from "@jest/globals";

import { ERenderAmbientOcclusionMethod, ERenderAntialiasing } from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_RENDER_FEATURE_CHOICE,
  resolveRenderFeatures,
  toRenderFeatureChoice,
} from "@/core/render/lib/settings/render-feature-choice";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";

describe("render presets", () => {
  it("starts a fresh install on Base", () => {
    expect(DEFAULT_RENDER_FEATURE_CHOICE.preset).toBe(ERenderPreset.BASE);
  });

  // A preset value past a bound would be held to it the moment it is stored back, and so never be what it says.
  it("sets every feature within the bounds a stored choice is held to", () => {
    for (const preset of Object.values(ERenderPreset)) {
      expect(resolveRenderFeatures(toRenderFeatureChoice({ overrides: RENDER_PRESETS[preset], preset }))).toEqual(
        RENDER_PRESETS[preset]
      );
    }
  });

  it("draws Enhanced at its best with TAA, short of ray reconstruction and traced occlusion", () => {
    const enhanced = RENDER_PRESETS[ERenderPreset.ENHANCED];

    expect(enhanced.antialiasing).toBe(ERenderAntialiasing.TAA);
    expect(enhanced.upscaling.isRayReconstructed).toBe(false);
    expect(enhanced.ambientOcclusion.method).not.toBe(ERenderAmbientOcclusionMethod.RTAO);
  });
});
