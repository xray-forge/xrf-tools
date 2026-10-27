import { formatNumber } from "@xrf/format";
import { ERenderWaterMode, RenderWaterMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/ChoiceFormRow/choice-form-row-option";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each water value is offered between. */
export const RENDER_WATER_LIMITS = {
  distortion: toRenderLimits(RENDER_FEATURE_SCHEMA.water.distortion, 0.005),
  reflection: toRenderLimits(RENDER_FEATURE_SCHEMA.water.reflection, 0.05),
  ripple: toRenderLimits(RENDER_FEATURE_SCHEMA.water.ripple, 0.05),
  waveHeight: toRenderLimits(RENDER_FEATURE_SCHEMA.water.waveHeight, 0.001),
  waveSpeed: toRenderLimits(RENDER_FEATURE_SCHEMA.water.waveSpeed, 1),
} as const;

/** The bounds each enhanced water value is offered between. */
export const RENDER_ENHANCED_WATER_LIMITS = {
  blurNoise: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.blurNoise, 0.05),
  calmFlow: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.calmFlow, 0.05),
  caustics: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.caustics, 0.01),
  flow: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.flow, 0.05),
  parallaxHeight: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.parallaxHeight, 0.005),
  reflectionBlur: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.reflectionBlur, 0.05),
  reflectivity: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.reflectivity, 0.05),
  refraction: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.refraction, 0.05),
  ripples: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.ripples, 0.05),
  softBorder: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.softBorder, 0.05),
  specular: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.specular, 0.1),
  turbidity: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.turbidity, 0.1),
  variation: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.variation, 0.05),
  vibrance: toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.vibrance, 0.05),
} as const;

/** @returns A wave's height, in centimetres, which is the size the engine's are. */
export function formatWaveHeight(height: number): string {
  return `${formatNumber(height * 100, 1)} cm`;
}

/** @returns How fast the waves travel, in the engine's own units. */
export function formatWaveSpeed(speed: number): string {
  return formatNumber(speed, 0);
}

/** @returns A multiple of the engine's own value. */
export function formatWaterMultiple(multiple: number): string {
  return `${formatNumber(multiple, 2)}×`;
}

/** @returns One of the enhanced water's strengths. */
export function formatWaterStrength(strength: number): string {
  return formatNumber(strength, 2);
}

/** @returns The depth the enhanced water's edge fades over. */
export function formatWaterBorder(depth: number): string {
  return `${formatNumber(depth, 2)} m`;
}

/** @returns How far the distortion moves what is behind the water and distorting particles, a share of the screen. */
export function formatWaterDistortion(distortion: number): string {
  return `${formatNumber(distortion * 100, 1)}%`;
}

const RENDER_WATER_MODE_DESCRIPTIONS: Readonly<Record<RenderWaterMode, string>> = {
  [ERenderWaterMode.ENGINE]: "Draws the game's rippled water, reflecting the sky.",
  [ERenderWaterMode.ENHANCED]:
    "Refracts and clouds what lies under the water by its depth, reflects the scene and softens the shore.",
};

/**
 * @param mode - A water mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderWaterMode(mode: RenderWaterMode): string {
  return RENDER_WATER_MODE_DESCRIPTIONS[mode];
}

/** The water modes, in the order they are offered. */
export const RENDER_WATER_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderWaterMode>> = [
  { label: "Engine", value: ERenderWaterMode.ENGINE },
  { label: "Enhanced", value: ERenderWaterMode.ENHANCED },
];
