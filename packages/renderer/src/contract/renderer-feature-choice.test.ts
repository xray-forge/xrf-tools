import { describe, expect, it } from "@jest/globals";

import { ERendererAmbientOcclusionQuality } from "#/contract/renderer-ambient-occlusion-quality";
import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";
import {
  DEFAULT_RENDERER_FEATURE_CHOICE,
  isRendererFeatureChoiceCustom,
  mergeRendererFeatureOverrides,
  resolveRendererFeatures,
  toRendererFeatureChoice,
  toRendererFeatureSettings,
} from "#/contract/renderer-feature-choice";
import { IRendererFeatureSettings } from "#/contract/renderer-feature-settings";
import { ERendererPreset, RENDERER_PRESETS } from "#/contract/renderer-preset";
import { ERendererRenderScale } from "#/contract/renderer-render-scale";

describe("renderer feature choices", () => {
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

  it("reads back stored ambient occlusion overrides, dropping a quality it has not and holding a radius to its least", () => {
    expect(
      toRendererFeatureChoice({
        overrides: { ambientOcclusion: { isEnabled: false, quality: "ultra", radius: -1, strength: 1.5 } },
        preset: "base",
      }).overrides.ambientOcclusion
    ).toEqual({ isEnabled: false, quality: ERendererAmbientOcclusionQuality.ULTRA, radius: 0.1, strength: 1.5 });
    expect(
      toRendererFeatureChoice({ overrides: { ambientOcclusion: { quality: "extreme" } }, preset: "base" }).overrides
        .ambientOcclusion
    ).toBeUndefined();
    expect(
      isRendererFeatureChoiceCustom({ overrides: { ambientOcclusion: { radius: 2 } }, preset: ERendererPreset.BASE })
    ).toBe(true);
  });

  it("reads back stored grass overrides within the engine's console bounds, a radius in whole metres", () => {
    expect(
      toRendererFeatureChoice({
        overrides: { grass: { density: 0, height: 9, isEnabled: false, radius: 60.4 } },
        preset: "base",
      }).overrides.grass
    ).toEqual({ density: 0.1, height: 2, isEnabled: false, radius: 60 });
    expect(isRendererFeatureChoiceCustom({ overrides: { grass: { radius: 49 } }, preset: ERendererPreset.BASE })).toBe(
      false
    );
    expect(RENDERER_PRESETS[ERendererPreset.EDITING].grass.isEnabled).toBe(false);
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
      isRendererFeatureChoiceCustom({
        overrides: { shadows: { cascades: [20, 40, 160] } },
        preset: ERendererPreset.BASE,
      })
    ).toBe(true);
    expect(
      isRendererFeatureChoiceCustom({
        overrides: { shadows: { cascades: [20, 40, 160, 480] } },
        preset: ERendererPreset.BASE,
      })
    ).toBe(false);
  });

  it("reads back stored upscaling overrides, dropping a scale it has not and holding a sharpening to one", () => {
    expect(
      toRendererFeatureChoice({ overrides: { upscaling: { scale: "quality", sharpening: 0.25 } }, preset: "base" })
        .overrides.upscaling
    ).toEqual({ scale: ERendererRenderScale.QUALITY, sharpening: 0.25 });
    expect(
      toRendererFeatureChoice({ overrides: { upscaling: { scale: "ultra", sharpening: 2 } }, preset: "base" }).overrides
        .upscaling
    ).toEqual({ sharpening: 1 });
  });

  // What arrives with a configure is checked once, there: everything past it takes the features as they are.
  it("holds arriving features to the schema, and takes Base's for any missing or unusable", () => {
    const base: IRendererFeatureSettings = RENDERER_PRESETS[ERendererPreset.BASE];
    const features: IRendererFeatureSettings = toRendererFeatureSettings({
      ...base,
      antialiasing: "msaa",
      grass: { ...base.grass, density: 5, height: 9, radius: 1000 },
      shadows: { ...base.shadows, blend: 0.5 },
    });

    expect(features.antialiasing).toBe(base.antialiasing);
    expect(features.grass).toEqual({ density: 0.99, height: 2, isEnabled: true, radius: 300 });
    expect(features.shadows.blend).toBe(0.4);
    expect(features.lod).toEqual(base.lod);
    expect(toRendererFeatureSettings(undefined)).toEqual(base);
  });

  it("merges changes into the overrides setting by setting, keeping the rest of each group", () => {
    expect(
      mergeRendererFeatureOverrides(
        { antialiasing: ERendererAntialiasing.TAA, grass: { density: 0.3, radius: 60 } },
        { grass: { radius: 80 }, shadows: { cascades: [10, 30] } }
      )
    ).toEqual({
      antialiasing: ERendererAntialiasing.TAA,
      grass: { density: 0.3, radius: 80 },
      shadows: { cascades: [10, 30] },
    });
  });
});
