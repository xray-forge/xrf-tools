import { describe, expect, it } from "@jest/globals";

import {
  DEFAULT_RENDERER_FEATURE_CHOICE,
  ERendererAmbientOcclusionQuality,
  ERendererAntialiasing,
  ERendererPreset,
  isRendererFeatureChoiceCustom,
  RENDERER_PRESETS,
  resolveRendererFeatures,
  toRendererFeatureChoice,
} from "#/contract/renderer-features";

describe("renderer features", () => {
  it("resolves a preset with nothing on top of it to the preset, and not as custom", () => {
    const choice = { overrides: {}, preset: ERendererPreset.EDITING };

    expect(resolveRendererFeatures(choice)).toEqual(RENDERER_PRESETS[ERendererPreset.EDITING]);
    expect(isRendererFeatureChoiceCustom(choice)).toBe(false);
  });

  it("puts what was changed over the preset, a threshold at a time", () => {
    const choice = {
      overrides: { antialiasing: ERendererAntialiasing.FXAA, lod: { ssaA: 80 } },
      preset: ERendererPreset.BASE,
    };
    const features = resolveRendererFeatures(choice);

    expect(features.antialiasing).toBe(ERendererAntialiasing.FXAA);
    expect(features.lod.ssaA).toBe(80);
    expect(features.lod.ssaB).toBe(RENDERER_PRESETS[ERendererPreset.BASE].lod.ssaB);
    expect(isRendererFeatureChoiceCustom(choice)).toBe(true);
  });

  // An override the preset already has is no change: choosing SMAA on Base is still Base.
  it("calls a choice custom only where it differs from its preset", () => {
    expect(
      isRendererFeatureChoiceCustom({
        overrides: { antialiasing: ERendererAntialiasing.SMAA, lod: { ssaA: 64 } },
        preset: ERendererPreset.BASE,
      })
    ).toBe(false);
  });

  it("reads back a stored choice, dropping whatever the features do not take", () => {
    expect(
      toRendererFeatureChoice({
        overrides: { antialiasing: "msaa", isGpuTimed: false, lod: { ssaA: Infinity, ssaB: 40, unknown: 1 } },
        preset: "editing",
      })
    ).toEqual({ overrides: { isGpuTimed: false, lod: { ssaB: 40 } }, preset: ERendererPreset.EDITING });
    expect(toRendererFeatureChoice("nonsense")).toBe(DEFAULT_RENDERER_FEATURE_CHOICE);
    expect(toRendererFeatureChoice({ preset: "ultra" }).preset).toBe(ERendererPreset.BASE);
  });

  it("reads back stored ambient occlusion overrides, dropping a quality it has not and a radius below zero", () => {
    expect(
      toRendererFeatureChoice({
        overrides: { ambientOcclusion: { isEnabled: false, quality: "ultra", radius: -1, strength: 1.5 } },
        preset: "base",
      }).overrides.ambientOcclusion
    ).toEqual({ isEnabled: false, quality: ERendererAmbientOcclusionQuality.ULTRA, strength: 1.5 });
    expect(
      toRendererFeatureChoice({ overrides: { ambientOcclusion: { quality: "extreme" } }, preset: "base" }).overrides
        .ambientOcclusion
    ).toBeUndefined();
    expect(
      isRendererFeatureChoiceCustom({ overrides: { ambientOcclusion: { radius: 2 } }, preset: ERendererPreset.BASE })
    ).toBe(true);
  });

  it("reads back stored shadow overrides, dropping a run of cascades past the limit and anything not a width", () => {
    expect(
      toRendererFeatureChoice({
        overrides: { shadows: { bias: 2, blend: 0.05, cascades: [20, 40], isStaggered: false, resolution: "big" } },
        preset: "base",
      }).overrides.shadows
    ).toEqual({ bias: 2, blend: 0.05, cascades: [20, 40], isStaggered: false });
    expect(
      toRendererFeatureChoice({ overrides: { shadows: { cascades: [1, 2, 3, 4, 5] } }, preset: "base" }).overrides
    ).toEqual({});
  });

  it("calls a changed cascade run custom, and a preset's own run not", () => {
    expect(
      isRendererFeatureChoiceCustom({ overrides: { shadows: { cascades: [20, 40] } }, preset: ERendererPreset.BASE })
    ).toBe(true);
    expect(
      isRendererFeatureChoiceCustom({
        overrides: { shadows: { cascades: [20, 40, 160] } },
        preset: ERendererPreset.BASE,
      })
    ).toBe(false);
  });
});
