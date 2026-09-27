import {
  DEFAULT_RENDERER_GRASS_SETTINGS,
  ERendererAmbientOcclusionQuality,
  ERendererAntialiasing,
  ERendererLightShadowFilter,
  ERendererRenderScale,
  IRendererNumberField,
  RENDERER_FEATURE_SCHEMA,
  RENDERER_SHADOW_CASCADE_WIDTHS,
} from "@xrf/renderer";

import { formatNumber } from "@/lib/format/number";

/** One value a choice offers, in display order: what Settings and the toolbar popovers both list. */
export interface IRenderChoiceOption<T extends string> {
  label: string;
  value: T;
}

/** The bounds a value is offered between, and the step it moves by. */
export interface IRenderLimits {
  min: number;
  max: number;
  step: number;
}

/**
 * @param field - What the renderer takes of a setting.
 * @param step - The step it is offered by.
 * @returns Its bounds as the renderer holds them, the step as offered.
 */
function toRenderLimits(field: IRendererNumberField, step: number): IRenderLimits {
  return { max: field.max, min: field.min, step };
}

/** How many cascades are offered, as counts. */
export const RENDER_SHADOW_CASCADE_OPTIONS: ReadonlyArray<IRenderChoiceOption<string>> =
  RENDERER_SHADOW_CASCADE_WIDTHS.map((_, index: number) => ({ label: String(index + 1), value: String(index + 1) }));

/** The map resolutions offered, in texels across. */
const RENDER_SHADOW_RESOLUTIONS: ReadonlyArray<number> = [1024, 2048, 4096];

export const RENDER_SHADOW_RESOLUTION_OPTIONS: ReadonlyArray<IRenderChoiceOption<string>> =
  RENDER_SHADOW_RESOLUTIONS.map((value: number) => ({ label: String(value), value: String(value) }));

/** The bounds each shadow value is offered between. */
export const RENDER_SHADOW_LIMITS = {
  bias: toRenderLimits(RENDERER_FEATURE_SCHEMA.shadows.bias, 0.25),
  blend: toRenderLimits(RENDERER_FEATURE_SCHEMA.shadows.blend, 0.01),
  filter: toRenderLimits(RENDERER_FEATURE_SCHEMA.shadows.filter, 1),
  reach: toRenderLimits(RENDERER_FEATURE_SCHEMA.shadows.reach, 50),
} as const;

/**
 * @param texels - Texels each way a shadow's edge is averaged over.
 * @returns It as both surfaces read it: a width, or the hardest edge for none.
 */
export function formatShadowFilter(texels: number): string {
  return texels ? `${texels} texel${texels > 1 ? "s" : ""}` : "Hard";
}

/**
 * @param blend - How far in from a cascade's edge the next is mixed in, as a share of its width.
 * @returns It as the settings read it: a percentage, or the engine's hard switch for none.
 */
export function formatCascadeBlend(blend: number): string {
  return blend > 0 ? `${Math.round(blend * 100)}%` : "Hard";
}

/** The engine's grass density, `r__detail_density`, which is a spacing: a smaller one plants more. */
const GAME_GRASS_DENSITY: number = DEFAULT_RENDERER_GRASS_SETTINGS.density;

/**
 * @param density - The engine's density, a spacing between tufts.
 * @returns How many times the game's density that is, as the settings offer it: higher is denser.
 */
export function toGrassDensityScale(density: number): number {
  return GAME_GRASS_DENSITY / density;
}

/**
 * @param scale - How many times the game's density the settings ask for.
 * @returns The engine's density that is.
 */
export function fromGrassDensityScale(scale: number): number {
  return GAME_GRASS_DENSITY / scale;
}

/**
 * The bounds each grass value is offered between, the engine console's: density as how many times the game's the tufts
 * stand, from its sparsest to its densest.
 */
export const RENDER_GRASS_LIMITS = {
  density: {
    max: toGrassDensityScale(RENDERER_FEATURE_SCHEMA.grass.density.min),
    min: toGrassDensityScale(RENDERER_FEATURE_SCHEMA.grass.density.max),
    step: 0.05,
  },
  height: toRenderLimits(RENDERER_FEATURE_SCHEMA.grass.height, 0.1),
  radius: toRenderLimits(RENDERER_FEATURE_SCHEMA.grass.radius, 1),
} as const;

/** @returns A grass density as the settings offer it: how many times the game's. */
export function formatGrassDensity(density: number): string {
  return `${formatNumber(toGrassDensityScale(density), 2)}×`;
}

/** @returns A grass radius, in metres. */
export function formatGrassRadius(radius: number): string {
  return `${formatNumber(radius, 0)} m`;
}

/** @returns A grass height, as the multiple of the game's it is. */
export function formatGrassHeight(height: number): string {
  return `${formatNumber(height, 1)}×`;
}

/** How far the upscaled frame's sharpening goes. */
export const RENDER_SHARPENING_LIMITS: IRenderLimits = toRenderLimits(
  RENDERER_FEATURE_SCHEMA.upscaling.sharpening,
  0.05
);

/** The bounds each ambient occlusion value is offered between. */
export const RENDER_AMBIENT_OCCLUSION_LIMITS = {
  radius: toRenderLimits(RENDERER_FEATURE_SCHEMA.ambientOcclusion.radius, 0.25),
  strength: toRenderLimits(RENDERER_FEATURE_SCHEMA.ambientOcclusion.strength, 0.1),
} as const;

/** @returns An occlusion radius, in metres. */
export function formatOcclusionRadius(radius: number): string {
  return `${formatNumber(radius, 2)} m`;
}

/** @returns An occlusion strength. */
export function formatOcclusionStrength(strength: number): string {
  return formatNumber(strength, 1);
}

/**
 * @param scale - A render scale.
 * @returns Its name as the settings say it, with the share of each side it draws.
 */
export function describeRenderScale(scale: ERendererRenderScale): string {
  switch (scale) {
    case ERendererRenderScale.NATIVE:
      return "Native";
    case ERendererRenderScale.QUALITY:
      return "Quality";
    case ERendererRenderScale.BALANCED:
      return "Balanced";
    case ERendererRenderScale.PERFORMANCE:
      return "Performance";
  }
}

/** The render scales, in the order they are offered. */
export const RENDER_SCALE_OPTIONS: ReadonlyArray<IRenderChoiceOption<ERendererRenderScale>> = Object.values(
  ERendererRenderScale
).map((value: ERendererRenderScale) => ({ label: describeRenderScale(value), value }));

/**
 * @param quality - An ambient occlusion quality.
 * @returns Its name as the settings say it.
 */
export function describeRenderAmbientOcclusionQuality(quality: ERendererAmbientOcclusionQuality): string {
  switch (quality) {
    case ERendererAmbientOcclusionQuality.LOW:
      return "Low";
    case ERendererAmbientOcclusionQuality.MEDIUM:
      return "Medium";
    case ERendererAmbientOcclusionQuality.HIGH:
      return "High";
    case ERendererAmbientOcclusionQuality.ULTRA:
      return "Ultra";
  }
}

/** The ambient occlusion qualities, in the order they are offered. */
export const RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS: ReadonlyArray<
  IRenderChoiceOption<ERendererAmbientOcclusionQuality>
> = Object.values(ERendererAmbientOcclusionQuality).map((value: ERendererAmbientOcclusionQuality) => ({
  label: describeRenderAmbientOcclusionQuality(value),
  value,
}));

/**
 * @param mode - An antialiasing mode.
 * @returns Its name as the settings say it.
 */
export function describeRenderAntialiasing(mode: ERendererAntialiasing): string {
  switch (mode) {
    case ERendererAntialiasing.NONE:
      return "None";
    case ERendererAntialiasing.FXAA:
      return "FXAA";
    case ERendererAntialiasing.SMAA:
      return "SMAA";
    case ERendererAntialiasing.TAA:
      return "TAA";
    case ERendererAntialiasing.FSR2:
      return "FSR 2";
  }
}

/** Every antialiasing mode, in the order they are offered. */
export const RENDER_ANTIALIASING_OPTIONS: ReadonlyArray<IRenderChoiceOption<ERendererAntialiasing>> = Object.values(
  ERendererAntialiasing
).map((value: ERendererAntialiasing) => ({ label: describeRenderAntialiasing(value), value }));

/**
 * @param filter - How a light's shadow is filtered.
 * @returns Its name as the settings say it.
 */
export function describeRenderLightShadowFilter(filter: ERendererLightShadowFilter): string {
  switch (filter) {
    case ERendererLightShadowFilter.ENGINE:
      return "Engine";
    case ERendererLightShadowFilter.SOFT:
      return "Soft";
  }
}

/** The filters a light's shadow is offered in, in the order they are offered. */
export const RENDER_LIGHT_SHADOW_FILTER_OPTIONS: ReadonlyArray<IRenderChoiceOption<ERendererLightShadowFilter>> =
  Object.values(ERendererLightShadowFilter).map((value: ERendererLightShadowFilter) => ({
    label: describeRenderLightShadowFilter(value),
    value,
  }));
