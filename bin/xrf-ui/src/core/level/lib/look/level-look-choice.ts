import { Nullable } from "@xrf/types";

import {
  ANOMALY_LEVEL_LOOK,
  ILevelImageCorrections,
  ILevelLightScales,
  ILevelLook,
  NEUTRAL_LEVEL_IMAGE_CORRECTIONS,
  NEUTRAL_LEVEL_LIGHT_SCALES,
  OPENXRAY_LEVEL_LOOK,
  toSettingsLevelLook,
} from "@/core/level/lib/look/level-look";
import { IRendererExposureSettings } from "@/core/render/lib/contract/renderer-exposure-settings";

/** Where a level's look comes from. */
export enum ELevelLookSource {
  /** What the open level's game ships, or the settings' where it ships nothing. */
  GAME = "game",
  /** The settings' exposure, nothing scaled or corrected. */
  SETTINGS = "settings",
  /** What Anomaly ships, whatever game is open. */
  ANOMALY = "anomaly",
  /** OpenXRay's own defaults, whatever game is open. */
  OPENXRAY = "openxray",
  /** Values of its own, from a preset or edited by hand. */
  CUSTOM = "custom",
}

/** A look saved under a name. */
export interface ILevelLookPreset {
  name: string;
  look: ILevelLook;
}

/**
 * What the level viewer's look is chosen as, kept over runs: where it comes from, its own values while custom, and the
 * presets saved.
 */
export interface ILevelLookChoice {
  source: ELevelLookSource;
  /** The values a custom look draws with. */
  custom: Nullable<ILevelLook>;
  /** The preset the custom look was taken from while it is unedited, or null. */
  preset: Nullable<string>;
  presets: ReadonlyArray<ILevelLookPreset>;
}

/** The game's own look, and no presets. */
export const DEFAULT_LEVEL_LOOK_CHOICE: ILevelLookChoice = {
  custom: null,
  preset: null,
  presets: [],
  source: ELevelLookSource.GAME,
};

/**
 * @param choice - How the look is chosen.
 * @param game - The open level's game's look, or null where it ships none or none is open.
 * @param exposure - The settings' exposure.
 * @returns The look the level is drawn with.
 */
export function resolveLevelLook(
  choice: ILevelLookChoice,
  game: Nullable<ILevelLook>,
  exposure: IRendererExposureSettings
): ILevelLook {
  switch (choice.source) {
    case ELevelLookSource.CUSTOM:
      return choice.custom ?? game ?? toSettingsLevelLook(exposure);

    case ELevelLookSource.GAME:
      return game ?? toSettingsLevelLook(exposure);

    case ELevelLookSource.SETTINGS:
      return toSettingsLevelLook(exposure);

    case ELevelLookSource.ANOMALY:
      return ANOMALY_LEVEL_LOOK;

    case ELevelLookSource.OPENXRAY:
      return OPENXRAY_LEVEL_LOOK;
  }
}

/**
 * Reads a stored choice, keeping only what is well formed, so a value from an older run or a hand edit cannot break
 * the viewer.
 *
 * @param stored - What local storage held, parsed.
 * @param exposure - The settings' exposure, for a stored look missing some of its own.
 * @returns The choice.
 */
export function toLevelLookChoice(stored: unknown, exposure: IRendererExposureSettings): ILevelLookChoice {
  if (!isRecord(stored)) {
    return DEFAULT_LEVEL_LOOK_CHOICE;
  }

  const source: ELevelLookSource = Object.values(ELevelLookSource).includes(stored.source as ELevelLookSource)
    ? (stored.source as ELevelLookSource)
    : ELevelLookSource.GAME;
  const custom: Nullable<ILevelLook> = isRecord(stored.custom) ? toLevelLook(stored.custom, exposure) : null;
  const presets: Array<ILevelLookPreset> = Array.isArray(stored.presets)
    ? stored.presets
        .filter((it: unknown) => isRecord(it) && typeof it.name === "string" && it.name.trim() && isRecord(it.look))
        .map((it: Record<string, unknown>) => ({
          look: toLevelLook(it.look as Record<string, unknown>, exposure),
          name: (it.name as string).trim(),
        }))
    : [];

  return {
    custom,
    preset: typeof stored.preset === "string" ? stored.preset : null,
    presets,
    source: source === ELevelLookSource.CUSTOM && !custom ? ELevelLookSource.GAME : source,
  };
}

function toLevelLook(stored: Record<string, unknown>, exposure: IRendererExposureSettings): ILevelLook {
  const storedExposure: Record<string, unknown> = isRecord(stored.exposure) ? stored.exposure : {};
  const scales: Record<string, unknown> = isRecord(stored.lightScales) ? stored.lightScales : {};
  const corrections: Record<string, unknown> = isRecord(stored.corrections) ? stored.corrections : {};
  const grading: Array<unknown> = Array.isArray(corrections.grading) ? corrections.grading : [];

  return {
    corrections: {
      exposure: toNumber(corrections.exposure, NEUTRAL_LEVEL_IMAGE_CORRECTIONS.exposure),
      gamma: toNumber(corrections.gamma, NEUTRAL_LEVEL_IMAGE_CORRECTIONS.gamma),
      grading: [toNumber(grading[0], 0), toNumber(grading[1], 0), toNumber(grading[2], 0)],
      saturation: toNumber(corrections.saturation, NEUTRAL_LEVEL_IMAGE_CORRECTIONS.saturation),
    } satisfies ILevelImageCorrections,
    exposure: {
      adaptation: toNumber(storedExposure.adaptation, exposure.adaptation),
      amount: toNumber(storedExposure.amount, exposure.amount),
      isEnabled: typeof storedExposure.isEnabled === "boolean" ? storedExposure.isEnabled : exposure.isEnabled,
      lowLuminance: toNumber(storedExposure.lowLuminance, exposure.lowLuminance),
      middleGray: toNumber(storedExposure.middleGray, exposure.middleGray),
    },
    lightScales: {
      ambient: toNumber(scales.ambient, NEUTRAL_LEVEL_LIGHT_SCALES.ambient),
      hemi: toNumber(scales.hemi, NEUTRAL_LEVEL_LIGHT_SCALES.hemi),
      sun: toNumber(scales.sun, NEUTRAL_LEVEL_LIGHT_SCALES.sun),
    } satisfies ILevelLightScales,
  };
}

function toNumber(value: unknown, fallback: number): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}
