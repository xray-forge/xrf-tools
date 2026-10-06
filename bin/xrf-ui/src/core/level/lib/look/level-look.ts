import { LevelConsoleDefaults } from "@/core/ipc/types/xrf-app";
import { EXrayEngine } from "@/core/ipc/types/xrf-engine-target";
import { DEFAULT_RENDER_EXPOSURE_SETTINGS } from "@/core/render/lib/settings/render-feature-defaults";
import { TRenderExposureSettings } from "@/core/render/lib/settings/render-feature-settings";

/** How a game's console scales the weather's light: `r2_sun_lumscale`, `_hemi` and `_amb`. */
export interface ILevelLightScales {
  sun: number;
  hemi: number;
  ambient: number;
}

/** Anomaly's `img_corrections` over the finished frame: `r__exposure`, `r__gamma`, `r__saturation`, `r__color_grading`. */
export interface ILevelImageCorrections {
  exposure: number;
  gamma: number;
  saturation: number;
  /** The colour mid tones are graded towards; black grades nothing. */
  grading: [number, number, number];
}

/** The engine's bloom: `r2_ls_bloom_threshold`, `r2_ls_bloom_kernel_g` and `r2_ls_bloom_kernel_scale`. */
export interface ILevelBloom {
  isEnabled: boolean;
  threshold: number;
  radius: number;
  strength: number;
}

/**
 * How a level is exposed, lit, bloomed and corrected: what the engine's console sets of it, rather than what the weather
 * does.
 */
export interface ILevelLook {
  exposure: TRenderExposureSettings;
  lightScales: ILevelLightScales;
  bloom: ILevelBloom;
  corrections: ILevelImageCorrections;
}

/** OpenXRay's console defaults, which bloom the whole frame. */
export const OPENXRAY_LEVEL_BLOOM: ILevelBloom = { isEnabled: true, radius: 3, strength: 0.7, threshold: 0.00001 };

/** Monolith's console defaults: the threshold at one, which holds the bloom back to the brightest. */
export const MONOLITH_LEVEL_BLOOM: ILevelBloom = { ...OPENXRAY_LEVEL_BLOOM, threshold: 1 };

/** No bloom: what the settings alone draw. */
export const NO_LEVEL_BLOOM: ILevelBloom = { ...OPENXRAY_LEVEL_BLOOM, isEnabled: false };

/** Nothing scaled, as the engine's own console leaves it. */
export const NEUTRAL_LEVEL_LIGHT_SCALES: ILevelLightScales = { ambient: 1, hemi: 1, sun: 1 };

/** Nothing corrected, as the engine's own console leaves it. */
export const NEUTRAL_LEVEL_IMAGE_CORRECTIONS: ILevelImageCorrections = {
  exposure: 1,
  gamma: 1,
  grading: [0, 0, 0],
  saturation: 1,
};

/**
 * What Anomaly ships in `default_controls.ltx`: the sun doubled, a tonemap brought to 1.5 floored at 0.5, and a bloom
 * too faint to see.
 */
export const ANOMALY_LEVEL_LOOK: ILevelLook = {
  bloom: { isEnabled: true, radius: 1, strength: 0.05, threshold: 0 },
  corrections: NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  exposure: { adaptation: 10, amount: 1, isEnabled: true, lowLuminance: 0.5, middleGray: 1.5 },
  lightScales: { ambient: 1, hemi: 1, sun: 2 },
};

/**
 * What XRF's render presets ship in `rspec_shared.ltx`: the hemisphere at 0.6 and the ambient at 0.1, a tonemap brought
 * to 1.1 floored at 0.4, and OpenXRay's bloom, which they leave alone.
 */
export const XRF_LEVEL_LOOK: ILevelLook = {
  bloom: OPENXRAY_LEVEL_BLOOM,
  corrections: NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  exposure: { adaptation: 0.2, amount: 1, isEnabled: true, lowLuminance: 0.4, middleGray: 1.1 },
  lightScales: { ambient: 0.1, hemi: 0.6, sun: 1 },
};

/** OpenXRay's own console defaults: its exposure and bloom, nothing scaled or corrected. */
export const OPENXRAY_LEVEL_LOOK: ILevelLook = {
  bloom: OPENXRAY_LEVEL_BLOOM,
  corrections: NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  exposure: DEFAULT_RENDER_EXPOSURE_SETTINGS,
  lightScales: NEUTRAL_LEVEL_LIGHT_SCALES,
};

/**
 * @param exposure - The exposure the settings give every viewport.
 * @returns The look the settings alone make: their exposure, nothing scaled, bloomed or corrected.
 */
export function toSettingsLevelLook(exposure: TRenderExposureSettings): ILevelLook {
  return {
    bloom: NO_LEVEL_BLOOM,
    corrections: NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
    exposure,
    lightScales: NEUTRAL_LEVEL_LIGHT_SCALES,
  };
}

/**
 * @param defaults - What the game's console defaults set, nothing where it ships none.
 * @param exposure - The settings' exposure, for whatever the game leaves to the engine.
 * @param engine - The engine the level is drawn as, whose own bloom stands for whatever the game leaves.
 * @returns The look the game ships its levels with.
 */
export function toGameLevelLook(
  defaults: LevelConsoleDefaults,
  exposure: TRenderExposureSettings,
  engine: EXrayEngine
): ILevelLook {
  const [r, g, b] = defaults.colorGrading ?? NEUTRAL_LEVEL_IMAGE_CORRECTIONS.grading;
  const bloom: ILevelBloom = engine === EXrayEngine.EXTENDED ? MONOLITH_LEVEL_BLOOM : OPENXRAY_LEVEL_BLOOM;

  return {
    bloom: {
      isEnabled: true,
      radius: defaults.bloomRadius ?? bloom.radius,
      strength: defaults.bloomStrength ?? bloom.strength,
      threshold: defaults.bloomThreshold ?? bloom.threshold,
    },
    corrections: {
      exposure: defaults.imageExposure ?? NEUTRAL_LEVEL_IMAGE_CORRECTIONS.exposure,
      gamma: defaults.imageGamma ?? NEUTRAL_LEVEL_IMAGE_CORRECTIONS.gamma,
      grading: [r ?? 0, g ?? 0, b ?? 0],
      saturation: defaults.imageSaturation ?? NEUTRAL_LEVEL_IMAGE_CORRECTIONS.saturation,
    },
    exposure: {
      adaptation: defaults.tonemapAdaptation ?? exposure.adaptation,
      amount: defaults.tonemapAmount ?? exposure.amount,
      isEnabled: defaults.isTonemapped ?? exposure.isEnabled,
      lowLuminance: defaults.tonemapLowLuminance ?? exposure.lowLuminance,
      middleGray: defaults.tonemapMiddleGray ?? exposure.middleGray,
    },
    lightScales: {
      ambient: defaults.ambientScale ?? NEUTRAL_LEVEL_LIGHT_SCALES.ambient,
      hemi: defaults.hemiScale ?? NEUTRAL_LEVEL_LIGHT_SCALES.hemi,
      sun: defaults.sunScale ?? NEUTRAL_LEVEL_LIGHT_SCALES.sun,
    },
  };
}
