import { ELevelLookFieldOff } from "@/applications/level-viewer/lib/look/level-look-field-off";
import { ILevelLook } from "@/core/level/lib/look/level-look";
import { RENDER_EXPOSURE_LIMITS } from "@/core/render/lib/features/render-exposure-choices";
import { TRenderEnhancedBloomSettings } from "@/core/render/lib/settings/render-feature-settings";

/** One number of the look a slider sets. */
export interface ILevelLookField {
  label: string;
  /** What leaves it off, where anything does. */
  offUnder?: ELevelLookFieldOff;
  min: number;
  max: number;
  step: number;
  read: (look: ILevelLook) => number;
  write: (look: ILevelLook, value: number) => ILevelLook;
}

/** One of the enhanced bloom's strengths a slider sets. */
export interface ILevelEnhancedBloomField {
  key: Exclude<keyof TRenderEnhancedBloomSettings, "mode">;
  label: string;
}

/** The enhanced bloom's sliders, in the order they are offered. */
export const LEVEL_ENHANCED_BLOOM_FIELDS: ReadonlyArray<ILevelEnhancedBloomField> = [
  { key: "threshold", label: "Threshold" },
  { key: "exposure", label: "Brightness" },
  { key: "blur", label: "Blur" },
  { key: "vibrance", label: "Vibrance" },
  { key: "sky", label: "Sky" },
];

/** The engine meter's sliders, which only the engine light model reads: the physical light meters by histogram. */
export const LEVEL_LOOK_TONEMAP_FIELDS: ReadonlyArray<ILevelLookField> = [
  {
    label: "Strength",
    offUnder: ELevelLookFieldOff.PHYSICAL_LIGHT,
    ...RENDER_EXPOSURE_LIMITS.amount,
    read: (look) => look.exposure.amount,
    write: (look, amount) => ({ ...look, exposure: { ...look.exposure, amount } }),
  },
  {
    label: "Middle gray",
    offUnder: ELevelLookFieldOff.PHYSICAL_LIGHT,
    ...RENDER_EXPOSURE_LIMITS.middleGray,
    read: (look) => look.exposure.middleGray,
    write: (look, middleGray) => ({ ...look, exposure: { ...look.exposure, middleGray } }),
  },
  {
    label: "Luminance floor",
    offUnder: ELevelLookFieldOff.PHYSICAL_LIGHT,
    ...RENDER_EXPOSURE_LIMITS.lowLuminance,
    read: (look) => look.exposure.lowLuminance,
    write: (look, lowLuminance) => ({ ...look, exposure: { ...look.exposure, lowLuminance } }),
  },
  {
    label: "Adaptation speed",
    offUnder: ELevelLookFieldOff.PHYSICAL_LIGHT,
    ...RENDER_EXPOSURE_LIMITS.adaptation,
    read: (look) => look.exposure.adaptation,
    write: (look, adaptation) => ({ ...look, exposure: { ...look.exposure, adaptation } }),
  },
];

/**
 * The light's scales: the sun's and the flat ambient's, which the physical light levels light by the atmosphere in
 * place of, and the hemisphere's.
 */
export const LEVEL_LOOK_LIGHT_SCALE_FIELDS: ReadonlyArray<ILevelLookField> = [
  {
    label: "Sun",
    offUnder: ELevelLookFieldOff.PHYSICAL_LEVELS,
    max: 4,
    min: 0,
    read: (look) => look.lightScales.sun,
    step: 0.05,
    write: (look, sun) => ({ ...look, lightScales: { ...look.lightScales, sun } }),
  },
  {
    label: "Hemisphere",
    max: 4,
    min: 0,
    read: (look) => look.lightScales.hemi,
    step: 0.05,
    write: (look, hemi) => ({ ...look, lightScales: { ...look.lightScales, hemi } }),
  },
  {
    label: "Ambient",
    offUnder: ELevelLookFieldOff.PHYSICAL_LEVELS,
    max: 4,
    min: 0,
    read: (look) => look.lightScales.ambient,
    step: 0.05,
    write: (look, ambient) => ({ ...look, lightScales: { ...look.lightScales, ambient } }),
  },
];

/** The engine bloom's sliders, which the enhanced bloom's replace. */
export const LEVEL_LOOK_BLOOM_FIELDS: ReadonlyArray<ILevelLookField> = [
  {
    label: "Threshold",
    max: 1,
    min: 0,
    read: (look) => look.bloom.threshold,
    step: 0.01,
    write: (look, threshold) => ({ ...look, bloom: { ...look.bloom, threshold } }),
  },
  {
    label: "Radius",
    max: 7,
    min: 1,
    read: (look) => look.bloom.radius,
    step: 0.1,
    write: (look, radius) => ({ ...look, bloom: { ...look.bloom, radius } }),
  },
  {
    label: "Strength",
    max: 2,
    min: 0.05,
    read: (look) => look.bloom.strength,
    step: 0.05,
    write: (look, strength) => ({ ...look, bloom: { ...look.bloom, strength } }),
  },
];

/** The engine's corrections of the finished frame, `img_corrections`. */
export const LEVEL_LOOK_IMAGE_FIELDS: ReadonlyArray<ILevelLookField> = [
  {
    label: "Exposure",
    max: 4,
    min: 0.5,
    read: (look) => look.corrections.exposure,
    step: 0.05,
    write: (look, exposure) => ({ ...look, corrections: { ...look.corrections, exposure } }),
  },
  {
    label: "Gamma",
    max: 2.2,
    min: 0.5,
    read: (look) => look.corrections.gamma,
    step: 0.05,
    write: (look, gamma) => ({ ...look, corrections: { ...look.corrections, gamma } }),
  },
  {
    label: "Saturation",
    max: 2,
    min: 0,
    read: (look) => look.corrections.saturation,
    step: 0.05,
    write: (look, saturation) => ({ ...look, corrections: { ...look.corrections, saturation } }),
  },
  ...(["Grading red", "Grading green", "Grading blue"] as const).map(
    (label: string, channel: number): ILevelLookField => ({
      label,
      max: 1,
      min: 0,
      read: (look) => look.corrections.grading[channel] ?? 0,
      step: 0.01,
      write: (look, value) => {
        const grading: [number, number, number] = [...look.corrections.grading];

        grading[channel] = value;

        return { ...look, corrections: { ...look.corrections, grading } };
      },
    })
  ),
];
