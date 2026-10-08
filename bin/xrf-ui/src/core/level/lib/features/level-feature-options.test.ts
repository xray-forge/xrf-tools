import { describe, expect, it } from "@jest/globals";

import {
  ERenderAmbientOcclusionQuality,
  ERenderAntialiasing,
  ERenderContactShadowMode,
  ERenderIndirectLightMode,
  ERenderLightShadowFilter,
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
  toLevelRendererAntialiasing,
  toLevelRendererFeature,
} from "@/core/level/lib/features/level-feature-options";
import { IRenderFeatureSettings } from "@/core/render/lib/settings/render-feature-settings";
import { ERenderPreset, RENDER_PRESETS } from "@/core/render/lib/settings/render-preset";
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
    ).toEqual({ ...mockLevelFeatureOptions(), grass: { radius: 300 } });
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
});
