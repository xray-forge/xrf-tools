import { describe, expect, it } from "@jest/globals";
import {
  ERendererAmbientOcclusionQuality,
  ERendererAntialiasing,
  ERendererLightShadowFilter,
  ERendererPreset,
  IRendererFeatureSettings,
  RENDERER_PRESETS,
} from "@xrf/renderer";

import {
  DEFAULT_LEVEL_FEATURE_OPTIONS,
  describeLevelFeatureToggle,
  ILevelFeatureOptions,
  LEVEL_ANTIALIASING_MODES,
  TLevelFeatureKey,
  toLevelFeatureOptions,
  toLevelFeatureView,
  toLevelRendererAntialiasing,
  toLevelRendererFeature,
} from "@/core/level/lib/features/level-feature-options";

const SETTINGS: IRendererFeatureSettings = RENDERER_PRESETS[ERendererPreset.BASE];

/** A view setting one value of every group over the settings. */
const VIEW: ILevelFeatureOptions = {
  ...DEFAULT_LEVEL_FEATURE_OPTIONS,
  ambientOcclusion: { quality: ERendererAmbientOcclusionQuality.LOW },
  grass: { radius: 80 },
  lights: { shadowFilter: ERendererLightShadowFilter.SOFT },
  shadows: { cascades: [20], filter: 0 },
};

const KEYS: ReadonlyArray<TLevelFeatureKey> = ["ambientOcclusion", "grass", "lights", "shadows"];

describe("level feature options", () => {
  it("follows the settings by default", () => {
    expect(toLevelRendererAntialiasing(ERendererAntialiasing.SMAA, DEFAULT_LEVEL_FEATURE_OPTIONS, true)).toBe(
      ERendererAntialiasing.SMAA
    );

    for (const key of KEYS) {
      expect(toLevelRendererFeature(key, SETTINGS, DEFAULT_LEVEL_FEATURE_OPTIONS, true)).toEqual(SETTINGS[key]);
    }
  });

  it("smooths with the view's own mode, which can be any but none", () => {
    const view: ILevelFeatureOptions = { ...DEFAULT_LEVEL_FEATURE_OPTIONS, antialiasing: ERendererAntialiasing.FXAA };

    expect(toLevelRendererAntialiasing(ERendererAntialiasing.SMAA, view, true)).toBe(ERendererAntialiasing.FXAA);
    expect(toLevelRendererAntialiasing(ERendererAntialiasing.SMAA, view, false)).toBe(ERendererAntialiasing.NONE);
    expect(LEVEL_ANTIALIASING_MODES).not.toContain(ERendererAntialiasing.NONE);
    expect(LEVEL_ANTIALIASING_MODES).toHaveLength(Object.values(ERendererAntialiasing).length - 1);
  });

  it("cannot smooth what the settings leave unsmoothed", () => {
    const view: ILevelFeatureOptions = { ...DEFAULT_LEVEL_FEATURE_OPTIONS, antialiasing: ERendererAntialiasing.SMAA };

    expect(toLevelRendererAntialiasing(ERendererAntialiasing.NONE, view, true)).toBe(ERendererAntialiasing.NONE);
  });

  it("draws every group with the view's own values over the settings'", () => {
    expect(toLevelRendererFeature("ambientOcclusion", SETTINGS, VIEW, true).quality).toBe(
      ERendererAmbientOcclusionQuality.LOW
    );
    expect(toLevelRendererFeature("grass", SETTINGS, VIEW, true)).toEqual({ ...SETTINGS.grass, radius: 80 });
    expect(toLevelRendererFeature("lights", SETTINGS, VIEW, true).shadowFilter).toBe(ERendererLightShadowFilter.SOFT);
    expect(toLevelRendererFeature("shadows", SETTINGS, VIEW, true)).toEqual({
      ...SETTINGS.shadows,
      cascades: [20],
      filter: 0,
    });
  });

  it("can turn every group off but not on", () => {
    for (const key of KEYS) {
      const off: IRendererFeatureSettings = { ...SETTINGS, [key]: { ...SETTINGS[key], isEnabled: false } };

      expect(toLevelRendererFeature(key, SETTINGS, VIEW, false).isEnabled).toBe(false);
      expect(toLevelRendererFeature(key, off, VIEW, true).isEnabled).toBe(false);
    }
  });

  it("resolves each group as the toolbar shows it: drawn as on, available as the settings allow", () => {
    const view = toLevelFeatureView({ ...SETTINGS, grass: { ...SETTINGS.grass, isEnabled: false } }, VIEW);

    expect(view.grass).toEqual({ isAvailable: false, value: { ...SETTINGS.grass, isEnabled: false, radius: 80 } });
    expect(view.shadows.isAvailable).toBe(true);
    expect(view.shadows.value.cascades).toEqual([20]);
  });

  it("says a toggle's state, and the settings' own where they keep it off", () => {
    const description = { isAvailable: true, isOn: true, isPlural: true, label: "Shadows", off: "Off", on: "On" };

    expect(describeLevelFeatureToggle(description)).toBe("On");
    expect(describeLevelFeatureToggle({ ...description, isOn: false })).toBe("Off");
    expect(describeLevelFeatureToggle({ ...description, isAvailable: false })).toBe(
      "Shadows are off in Settings, under Rendering"
    );
    expect(describeLevelFeatureToggle({ ...description, isAvailable: false, isPlural: false, label: "Grass" })).toBe(
      "Grass is off in Settings, under Rendering"
    );
  });

  // Stored by an earlier run, which may have known other settings or other bounds.
  it("reads a stored view back as the view set it, held to the renderer's bounds, the rest dropped", () => {
    expect(toLevelFeatureOptions(JSON.parse(JSON.stringify(VIEW)))).toEqual(VIEW);
    expect(
      toLevelFeatureOptions({
        antialiasing: ERendererAntialiasing.NONE,
        grass: { isEnabled: false, radius: 5000 },
        lights: { shadowFilter: "anomaly" },
        lod: { ssaA: 20 },
      })
    ).toEqual({ ...DEFAULT_LEVEL_FEATURE_OPTIONS, grass: { radius: 300 } });
    expect(toLevelFeatureOptions(null)).toEqual(DEFAULT_LEVEL_FEATURE_OPTIONS);
  });
});
