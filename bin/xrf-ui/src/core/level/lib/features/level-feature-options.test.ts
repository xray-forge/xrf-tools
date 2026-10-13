import { describe, expect, it } from "@jest/globals";

import {
  ERenderAmbientOcclusionQuality,
  ERenderAntialiasing,
  ERenderBloomMode,
  ERenderContactShadowMode,
  ERenderDebandingMode,
  ERenderDebandingQuality,
  ERenderFogMode,
  ERenderGlassMode,
  ERenderIndirectLightMode,
  ERenderLightShadowFilter,
  ERenderReflectionMode,
  ERenderVolumetricMode,
  ERenderWaterMode,
} from "@/core/ipc/types/xrf-renderer";
import {
  describeLevelFeatureToggle,
  ILevelFeatureOptions,
  LEVEL_ANTIALIASING_MODES,
  TLevelFeatureKey,
  toLevelFeatureOptions,
  toLevelFeatureView,
  toLevelIndirectLight,
  toLevelModeFeature,
  toLevelReflections,
  toLevelRendererAntialiasing,
  toLevelRendererFeature,
} from "@/core/level/lib/features/level-feature-options";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
import { ERenderResolution } from "@/core/render/lib/settings/render-resolution";
import { mockLevelFeatureOptions } from "@/fixtures/mocks/level.mocks";

const SETTINGS: IRenderFeatureSettings = RENDER_PRESETS[ERenderPreset.BASE];

const VIEW: ILevelFeatureOptions = {
  ...mockLevelFeatureOptions(),
  ambientOcclusion: { quality: ERenderAmbientOcclusionQuality.LOW },
  grass: { radius: 80 },
  lights: { shadowFilter: ERenderLightShadowFilter.SOFT },
  shadows: { cascades: [20], filter: 0 },
};

const KEYS: ReadonlyArray<TLevelFeatureKey> = ["ambientOcclusion", "grass", "lights", "shadows"];

describe("level feature options", () => {
  it("follows the settings by default", () => {
    expect(toLevelRendererAntialiasing(ERenderAntialiasing.SMAA, mockLevelFeatureOptions(), true)).toBe(
      ERenderAntialiasing.SMAA
    );

    for (const key of KEYS) {
      expect(toLevelRendererFeature(key, SETTINGS, mockLevelFeatureOptions(), true)).toEqual(SETTINGS[key]);
    }
  });

  it("smooths with the view's own mode, which can be any but none", () => {
    const view: ILevelFeatureOptions = { ...mockLevelFeatureOptions(), antialiasing: ERenderAntialiasing.FXAA };

    expect(toLevelRendererAntialiasing(ERenderAntialiasing.SMAA, view, true)).toBe(ERenderAntialiasing.FXAA);
    expect(toLevelRendererAntialiasing(ERenderAntialiasing.SMAA, view, false)).toBe(ERenderAntialiasing.NONE);
    expect(LEVEL_ANTIALIASING_MODES).not.toContain(ERenderAntialiasing.NONE);
    expect(LEVEL_ANTIALIASING_MODES).toHaveLength(Object.values(ERenderAntialiasing).length - 1);
  });

  it("cannot smooth what the settings leave unsmoothed", () => {
    const view: ILevelFeatureOptions = { ...mockLevelFeatureOptions(), antialiasing: ERenderAntialiasing.SMAA };

    expect(toLevelRendererAntialiasing(ERenderAntialiasing.NONE, view, true)).toBe(ERenderAntialiasing.NONE);
  });

  it("draws every group with the view's own values over the settings'", () => {
    expect(toLevelRendererFeature("ambientOcclusion", SETTINGS, VIEW, true).quality).toBe(
      ERenderAmbientOcclusionQuality.LOW
    );
    expect(toLevelRendererFeature("grass", SETTINGS, VIEW, true)).toEqual({ ...SETTINGS.grass, radius: 80 });
    expect(toLevelRendererFeature("lights", SETTINGS, VIEW, true).shadowFilter).toBe(ERenderLightShadowFilter.SOFT);
    expect(toLevelRendererFeature("shadows", SETTINGS, VIEW, true)).toEqual({
      ...SETTINGS.shadows,
      cascades: [20],
      filter: 0,
    });
  });

  it("can turn every group off but not on", () => {
    for (const key of KEYS) {
      const off: IRenderFeatureSettings = { ...SETTINGS, [key]: { ...SETTINGS[key], isEnabled: false } };

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
        antialiasing: ERenderAntialiasing.NONE,
        grass: { isEnabled: false, radius: 5000 },
        lights: { shadowFilter: "anomaly" },
        lod: { ssaA: 20 },
      })
    ).toEqual({ ...mockLevelFeatureOptions(), grass: { radius: 450 } });
    expect(toLevelFeatureOptions(null)).toEqual(mockLevelFeatureOptions());
  });

  it("keeps the water's mode and its enhanced strengths, held to their bounds, and drops a mode it does not know", () => {
    expect(
      toLevelFeatureOptions({
        water: { enhanced: { softBorder: 0.5, turbidity: 40 }, mode: ERenderWaterMode.ENHANCED },
      }).water
    ).toEqual({ enhanced: { softBorder: 0.5, turbidity: 10 }, mode: ERenderWaterMode.ENHANCED });
    expect(toLevelFeatureOptions({ water: { enhanced: { refraction: 1.2 }, mode: "ocean" } }).water).toEqual({
      enhanced: { refraction: 1.2 },
    });
  });

  it("keeps the contact shadows' own values, held to their bounds, over the settings' others", () => {
    const view: ILevelFeatureOptions = toLevelFeatureOptions({
      shadows: { contact: { length: 9, mode: ERenderContactShadowMode.ENGINE } },
    });

    expect(view.shadows).toEqual({ contact: { length: 4, mode: ERenderContactShadowMode.ENGINE } });
    expect(toLevelRendererFeature("shadows", SETTINGS, view, true).contact).toEqual({
      ...SETTINGS.shadows.contact,
      length: 4,
      mode: ERenderContactShadowMode.ENGINE,
    });
  });

  it("keeps the indirect light's own mode and intensity, held to their bounds, over the settings'", () => {
    const view: ILevelFeatureOptions = toLevelFeatureOptions({
      indirectLight: { intensity: 9, mode: ERenderIndirectLightMode.ENGINE, steps: 3 },
    });

    expect(view.indirectLight).toEqual({ intensity: 4, mode: ERenderIndirectLightMode.ENGINE });
    expect(toLevelIndirectLight(SETTINGS, mockLevelFeatureOptions())).toEqual(SETTINGS.indirectLight);
    expect(toLevelIndirectLight(SETTINGS, view)).toEqual({
      ...SETTINGS.indirectLight,
      intensity: 4,
      mode: ERenderIndirectLightMode.ENGINE,
    });
    expect(toLevelFeatureOptions({ indirectLight: { mode: "radiosity" } }).indirectLight).toEqual({});
  });

  it("keeps the reflections' own mode and strengths, held to their bounds, over the settings'", () => {
    const view: ILevelFeatureOptions = toLevelFeatureOptions({
      reflections: { distance: 900, intensity: 7, mode: ERenderReflectionMode.ENGINE, steps: 12 },
    });

    expect(view.reflections).toEqual({ distance: 200, intensity: 4, mode: ERenderReflectionMode.ENGINE });
    expect(toLevelReflections(SETTINGS, mockLevelFeatureOptions())).toEqual(SETTINGS.reflections);
    expect(toLevelReflections(SETTINGS, view)).toEqual({
      ...SETTINGS.reflections,
      distance: 200,
      intensity: 4,
      mode: ERenderReflectionMode.ENGINE,
    });
    expect(toLevelFeatureOptions({ reflections: { mode: "raytraced" } }).reflections).toEqual({});
  });

  it("keeps the fog's, the sky debanding's, the glass's and the bloom's own modes and strengths, held to their bounds", () => {
    const view: ILevelFeatureOptions = toLevelFeatureOptions({
      debanding: { mode: ERenderDebandingMode.ENGINE, quality: ERenderDebandingQuality.ULTRA, radius: 400 },
      enhancedBloom: { lens: 2, mode: ERenderBloomMode.ENHANCED, threshold: 0.5 },
      fog: { density: 9, height: 6, mode: ERenderFogMode.ENGINE, steps: 3 },
      glass: { blur: 1, mode: ERenderGlassMode.ENGINE, refraction: 5 },
    });

    expect(view.fog).toEqual({ density: 5, height: 6, mode: ERenderFogMode.ENGINE });
    expect(view.glass).toEqual({ mode: ERenderGlassMode.ENGINE, refraction: 2 });
    expect(toLevelModeFeature("glass", SETTINGS, view)).toEqual({ mode: ERenderGlassMode.ENGINE, refraction: 2 });
    expect(view.debanding).toEqual({
      mode: ERenderDebandingMode.ENGINE,
      quality: ERenderDebandingQuality.ULTRA,
      radius: 128,
    });
    expect(view.enhancedBloom).toEqual({ mode: ERenderBloomMode.ENHANCED, threshold: 1 });
    expect(toLevelModeFeature("fog", SETTINGS, mockLevelFeatureOptions())).toEqual(SETTINGS.fog);
    expect(toLevelModeFeature("fog", SETTINGS, view)).toEqual({
      ...SETTINGS.fog,
      density: 5,
      height: 6,
      mode: ERenderFogMode.ENGINE,
    });
    expect(toLevelModeFeature("enhancedBloom", SETTINGS, view)).toEqual({
      ...SETTINGS.enhancedBloom,
      mode: ERenderBloomMode.ENHANCED,
      threshold: 1,
    });
    expect(toLevelFeatureOptions({ fog: { mode: "volumetric" } }).fog).toEqual({});
  });

  it("keeps the volumetric lighting's own mode and strengths, held to their bounds", () => {
    const view: ILevelFeatureOptions = toLevelFeatureOptions({
      volumetric: { anisotropy: 2, distance: 10, mode: ERenderVolumetricMode.ENGINE, steps: 64 },
    });

    expect(view.volumetric).toEqual({ anisotropy: 0.9, distance: 25, mode: ERenderVolumetricMode.ENGINE });
    expect(toLevelModeFeature("volumetric", SETTINGS, view)).toEqual({
      ...SETTINGS.volumetric,
      anisotropy: 0.9,
      distance: 25,
      mode: ERenderVolumetricMode.ENGINE,
    });
    expect(toLevelModeFeature("volumetric", SETTINGS, mockLevelFeatureOptions())).toEqual(SETTINGS.volumetric);
    expect(toLevelFeatureOptions({ volumetric: { mode: "froxel" } }).volumetric).toEqual({});
  });

  it("keeps a view's own output resolution and scale, dropping either that the settings do not offer", () => {
    expect(toLevelFeatureOptions({ output: { resolution: "1440", scale: 1.5 } }).output).toEqual({
      resolution: ERenderResolution.HEIGHT_1440,
      scale: 1.5,
    });
    expect(toLevelFeatureOptions({ output: { resolution: "900", scale: 3 } }).output).toEqual({});
  });
});
