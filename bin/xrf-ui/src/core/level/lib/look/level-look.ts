import { LevelConsoleDefaults } from "@/core/ipc/types/xrf-app";
import {
  DEFAULT_RENDERER_EXPOSURE_SETTINGS,
  IRendererExposureSettings,
} from "@/core/render/lib/contract/renderer-exposure-settings";

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

/**
 * How a level is exposed, lit and corrected: what the engine's console sets of it, rather than what the weather does.
 */
export interface ILevelLook {
  exposure: IRendererExposureSettings;
  lightScales: ILevelLightScales;
  corrections: ILevelImageCorrections;
}

/** Nothing scaled, as the engine's own console leaves it. */
export const NEUTRAL_LEVEL_LIGHT_SCALES: ILevelLightScales = { ambient: 1, hemi: 1, sun: 1 };

/** Nothing corrected, as the engine's own console leaves it. */
export const NEUTRAL_LEVEL_IMAGE_CORRECTIONS: ILevelImageCorrections = {
  exposure: 1,
  gamma: 1,
  grading: [0, 0, 0],
  saturation: 1,
};

/** What Anomaly ships in `default_controls.ltx`: the sun doubled, and a tonemap brought to 1.5 floored at 0.5. */
export const ANOMALY_LEVEL_LOOK: ILevelLook = {
  corrections: NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  exposure: { adaptation: 10, amount: 1, isEnabled: true, lowLuminance: 0.5, middleGray: 1.5 },
  lightScales: { ambient: 1, hemi: 1, sun: 2 },
};

/** OpenXRay's own console defaults: its exposure, nothing scaled or corrected. */
export const OPENXRAY_LEVEL_LOOK: ILevelLook = {
  corrections: NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  exposure: DEFAULT_RENDERER_EXPOSURE_SETTINGS,
  lightScales: NEUTRAL_LEVEL_LIGHT_SCALES,
};

/**
 * @param exposure - The exposure the settings give every viewport.
 * @returns The look the settings alone make: their exposure, nothing scaled or corrected.
 */
export function toSettingsLevelLook(exposure: IRendererExposureSettings): ILevelLook {
  return { corrections: NEUTRAL_LEVEL_IMAGE_CORRECTIONS, exposure, lightScales: NEUTRAL_LEVEL_LIGHT_SCALES };
}

/**
 * @param defaults - What the game's console defaults set.
 * @param exposure - The settings' exposure, for whatever the game leaves to the engine.
 * @returns The look the game ships its levels with.
 */
export function toGameLevelLook(defaults: LevelConsoleDefaults, exposure: IRendererExposureSettings): ILevelLook {
  const [r, g, b] = defaults.colorGrading ?? NEUTRAL_LEVEL_IMAGE_CORRECTIONS.grading;

  return {
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
