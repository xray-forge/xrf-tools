import { describe, expect, it } from "@jest/globals";

import {
  ERendererAntialiasing,
  ERendererPreset,
  ERendererRenderScale,
  IRendererFeatureSettings,
  RENDERER_MAX_SHADOW_CASCADES,
  RENDERER_PRESETS,
} from "#/contract/renderer-features";
import { toFramePlan } from "#/graph/frame-plan";

const BASE: IRendererFeatureSettings = RENDERER_PRESETS[ERendererPreset.BASE];

/** `Base` with another mode, drawn at a scale, sharpened as much. */
function toFeatures(
  antialiasing: ERendererAntialiasing,
  scale: ERendererRenderScale = ERendererRenderScale.NATIVE,
  sharpening: number = 0.5
): IRendererFeatureSettings {
  return { ...BASE, antialiasing, upscaling: { scale, sharpening } };
}

describe("the frame plan", () => {
  it("smooths `Base` with SMAA alone, at the output's size", () => {
    expect(toFramePlan(BASE)).toMatchObject({
      isSpatial: false,
      resolve: null,
      sharpen: null,
      smoothing: ERendererAntialiasing.SMAA,
      upscale: 1,
    });
  });

  it("upscales in the resolve under the temporal modes, and denoises in RCAS under FSR 2 alone", () => {
    expect(toFramePlan(toFeatures(ERendererAntialiasing.TAA, ERendererRenderScale.PERFORMANCE))).toMatchObject({
      isSpatial: false,
      resolve: ERendererAntialiasing.TAA,
      sharpen: { isDenoised: false },
      smoothing: null,
      upscale: 2,
    });
    expect(toFramePlan(toFeatures(ERendererAntialiasing.FSR2, ERendererRenderScale.QUALITY))).toMatchObject({
      isSpatial: false,
      resolve: ERendererAntialiasing.FSR2,
      sharpen: { isDenoised: true },
    });
  });

  it("upscales with FSR 1 under every other mode, none included", () => {
    for (const mode of [ERendererAntialiasing.NONE, ERendererAntialiasing.FXAA, ERendererAntialiasing.SMAA]) {
      expect(toFramePlan(toFeatures(mode, ERendererRenderScale.BALANCED))).toMatchObject({
        isSpatial: true,
        resolve: null,
        sharpen: { isDenoised: false },
      });
    }
  });

  it("sharpens nothing drawn at the output's size, nor with the sharpening at none", () => {
    expect(toFramePlan(toFeatures(ERendererAntialiasing.FSR2)).sharpen).toBeNull();
    expect(toFramePlan(toFeatures(ERendererAntialiasing.TAA, ERendererRenderScale.QUALITY, 0)).sharpen).toBeNull();
  });

  it("draws at most the cascades the frame holds, and none while the sun's shadow is off", () => {
    const cascades: Array<number> = Array.from({ length: RENDERER_MAX_SHADOW_CASCADES + 2 }, (_, index) => index + 1);

    expect(toFramePlan({ ...BASE, shadows: { ...BASE.shadows, cascades } }).shadows?.count).toBe(
      RENDERER_MAX_SHADOW_CASCADES
    );
    expect(toFramePlan({ ...BASE, shadows: { ...BASE.shadows, isEnabled: false } }).shadows).toBeNull();
    expect(toFramePlan({ ...BASE, shadows: { ...BASE.shadows, cascades: [] } }).shadows).toBeNull();
  });

  it("shadows the lights only while they are on", () => {
    const lights = { ...BASE.lights, isEnabled: false, isShadowed: true };

    expect(toFramePlan({ ...BASE, lights })).toMatchObject({ isLightShadowed: false, isLit: false });
    expect(toFramePlan(RENDERER_PRESETS[ERendererPreset.EDITING])).toMatchObject({
      ambientOcclusion: null,
      isGrassy: false,
      isLightShadowed: false,
      isLit: true,
    });
  });

  it("culls what the depth hides while the features do, and drops its passes while they do not", () => {
    expect(toFramePlan(BASE).isOccluding).toBe(true);
    expect(toFramePlan({ ...BASE, isOcclusionCulled: false }).isOccluding).toBe(false);
  });
});
