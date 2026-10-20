import { ILevelLook } from "@/core/level/lib/look/level-look";
import { TRenderEnhancedBloomSettings } from "@/core/render/lib/settings/render-feature-settings";

/** One number of the look a slider sets. */
export interface ILevelLookField {
  label: string;
  min: number;
  max: number;
  step: number;
  read: (look: ILevelLook) => number;
  write: (look: ILevelLook, value: number) => ILevelLook;
}

/** The title of the group of the engine bloom's sliders, which the enhanced bloom's replace. */
export const LEVEL_LOOK_BLOOM_GROUP: string = "Bloom";

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

/** A titled group of the look's sliders. */
export interface ILevelLookFieldGroup {
  title: string;
  /** Whether only the engine light model reads it: the physical light meters its exposure by histogram. */
  isEngineOnly: boolean;
  fields: ReadonlyArray<ILevelLookField>;
}

/** The look's sliders, by the group they sit under. */
export const LEVEL_LOOK_FIELD_GROUPS: ReadonlyArray<ILevelLookFieldGroup> = [
  {
    fields: [
      {
        label: "Strength",
        max: 1,
        min: 0,
        read: (look) => look.exposure.amount,
        step: 0.01,
        write: (look, amount) => ({ ...look, exposure: { ...look.exposure, amount } }),
      },
      {
        label: "Middle gray",
        max: 2,
        min: 0,
        read: (look) => look.exposure.middleGray,
        step: 0.05,
        write: (look, middleGray) => ({ ...look, exposure: { ...look.exposure, middleGray } }),
      },
      {
        label: "Luminance floor",
        max: 1,
        min: 0.0001,
        read: (look) => look.exposure.lowLuminance,
        step: 0.01,
        write: (look, lowLuminance) => ({ ...look, exposure: { ...look.exposure, lowLuminance } }),
      },
      {
        label: "Adaptation speed",
        max: 10,
        min: 0.01,
        read: (look) => look.exposure.adaptation,
        step: 0.01,
        write: (look, adaptation) => ({ ...look, exposure: { ...look.exposure, adaptation } }),
      },
    ],
    isEngineOnly: true,
    title: "Tonemap",
  },
  {
    fields: [
      {
        label: "Sun",
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
        max: 4,
        min: 0,
        read: (look) => look.lightScales.ambient,
        step: 0.05,
        write: (look, ambient) => ({ ...look, lightScales: { ...look.lightScales, ambient } }),
      },
    ],
    isEngineOnly: false,
    title: "Light scales",
  },
  {
    fields: [
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
    ],
    isEngineOnly: false,
    title: LEVEL_LOOK_BLOOM_GROUP,
  },
  {
    fields: [
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
    ],
    isEngineOnly: false,
    title: "Image",
  },
];
