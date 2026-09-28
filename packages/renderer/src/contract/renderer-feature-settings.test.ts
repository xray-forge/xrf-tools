import { describe, expect, it } from "@jest/globals";

import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import { toRendererUpscale } from "#/contract/renderer-feature-settings";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { ERendererRenderScale } from "#/contract/renderer-render-scale";
import { isRendererSmoothing } from "#/contract/renderer-smoothing-antialiasing";
import { isRendererTemporal } from "#/contract/renderer-temporal-antialiasing";

describe("renderer features", () => {
  it("upscales by the render scale whatever the mode, and jitters for the temporal modes alone", () => {
    const base = RENDERER_PRESETS[ERendererPreset.BASE];
    const upscaling = { scale: ERendererRenderScale.PERFORMANCE, sharpening: 0.5 };

    expect(toRendererUpscale({ ...base, antialiasing: ERendererAntialiasing.SMAA, upscaling })).toBe(2);
    expect(toRendererUpscale(base)).toBe(1);
    expect(Object.values(ERendererAntialiasing).filter(isRendererTemporal)).toEqual([
      ERendererAntialiasing.TAA,
      ERendererAntialiasing.FSR2,
    ]);
    expect(Object.values(ERendererAntialiasing).filter(isRendererSmoothing)).toEqual([
      ERendererAntialiasing.FXAA,
      ERendererAntialiasing.SMAA,
    ]);
  });
});
