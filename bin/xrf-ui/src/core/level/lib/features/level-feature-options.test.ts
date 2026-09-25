import { describe, expect, it } from "@jest/globals";
import {
  DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDERER_SHADOW_SETTINGS,
  ERendererAmbientOcclusionQuality,
  ERendererAntialiasing,
} from "@xrf/renderer";

import {
  DEFAULT_LEVEL_FEATURE_OPTIONS,
  toLevelRendererAmbientOcclusion,
  toLevelRendererAntialiasing,
  toLevelRendererShadows,
} from "@/core/level/lib/features/level-feature-options";

describe("level feature options", () => {
  it("follows the settings by default", () => {
    expect(toLevelRendererAntialiasing(ERendererAntialiasing.SMAA, DEFAULT_LEVEL_FEATURE_OPTIONS, true)).toBe(
      ERendererAntialiasing.SMAA
    );
    expect(toLevelRendererShadows(DEFAULT_RENDERER_SHADOW_SETTINGS, DEFAULT_LEVEL_FEATURE_OPTIONS, true)).toEqual(
      DEFAULT_RENDERER_SHADOW_SETTINGS
    );
  });

  it("smooths with the view's own mode", () => {
    const view = { ...DEFAULT_LEVEL_FEATURE_OPTIONS, antialiasing: ERendererAntialiasing.FXAA };

    expect(toLevelRendererAntialiasing(ERendererAntialiasing.SMAA, view, true)).toBe(ERendererAntialiasing.FXAA);
    expect(toLevelRendererAntialiasing(ERendererAntialiasing.SMAA, view, false)).toBe(ERendererAntialiasing.NONE);
  });

  it("cannot smooth what the settings leave unsmoothed", () => {
    const view = { ...DEFAULT_LEVEL_FEATURE_OPTIONS, antialiasing: ERendererAntialiasing.SMAA };

    expect(toLevelRendererAntialiasing(ERendererAntialiasing.NONE, view, true)).toBe(ERendererAntialiasing.NONE);
  });

  it("draws shadows with the view's own values over the settings'", () => {
    const view = { ...DEFAULT_LEVEL_FEATURE_OPTIONS, shadows: { cascades: [20], filter: 0 } };

    expect(toLevelRendererShadows(DEFAULT_RENDERER_SHADOW_SETTINGS, view, true)).toEqual({
      ...DEFAULT_RENDERER_SHADOW_SETTINGS,
      cascades: [20],
      filter: 0,
    });
  });

  it("draws ambient occlusion with the view's own values over the settings', and can turn it off but not on", () => {
    const view = {
      ...DEFAULT_LEVEL_FEATURE_OPTIONS,
      ambientOcclusion: { quality: ERendererAmbientOcclusionQuality.LOW },
    };
    const off = { ...DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS, isEnabled: false };

    expect(toLevelRendererAmbientOcclusion(DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS, view, true)).toEqual({
      ...DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS,
      quality: ERendererAmbientOcclusionQuality.LOW,
    });
    expect(toLevelRendererAmbientOcclusion(DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS, view, false).isEnabled).toBe(
      false
    );
    expect(toLevelRendererAmbientOcclusion(off, view, true).isEnabled).toBe(false);
  });

  it("can turn shadows off but not on", () => {
    const off = { ...DEFAULT_RENDERER_SHADOW_SETTINGS, isEnabled: false };

    expect(
      toLevelRendererShadows(DEFAULT_RENDERER_SHADOW_SETTINGS, DEFAULT_LEVEL_FEATURE_OPTIONS, false).isEnabled
    ).toBe(false);
    expect(toLevelRendererShadows(off, DEFAULT_LEVEL_FEATURE_OPTIONS, true).isEnabled).toBe(false);
  });
});
