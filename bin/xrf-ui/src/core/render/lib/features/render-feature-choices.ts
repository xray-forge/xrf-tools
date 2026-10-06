import {
  ERenderAmbientOcclusionQuality,
  ERenderAntialiasing,
  ERenderLightShadowFilter,
  ERenderScale,
  RenderAmbientOcclusionQuality,
  RenderAntialiasing,
  RenderLightShadowFilter,
  RenderScale,
} from "@/core/ipc/types/xrf-renderer";
import {
  DEFAULT_RENDER_GRASS_SETTINGS,
  RENDER_SHADOW_CASCADE_WIDTHS,
} from "@/core/render/lib/settings/render-feature-defaults";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { FRAME_RATE_LIMITS, TFrameRateLimit } from "@/core/render/lib/settings/render-frame-rate-limit";
import { ERenderResolution, RENDER_RESOLUTIONS } from "@/core/render/lib/settings/render-resolution";
import { IRenderNumberField } from "@/core/render/lib/settings/render-setting-field";
import { formatNumber, formatPercent } from "@/lib/format/number";

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
function toRenderLimits(field: IRenderNumberField, step: number): IRenderLimits {
  return { max: field.max, min: field.min, step };
}

/** How many cascades are offered, as counts. */
export const RENDER_SHADOW_CASCADE_OPTIONS: ReadonlyArray<IRenderChoiceOption<string>> =
  RENDER_SHADOW_CASCADE_WIDTHS.map((_, index: number) => ({ label: String(index + 1), value: String(index + 1) }));

/** The map resolutions offered, in texels across. */
const RENDER_SHADOW_RESOLUTIONS: ReadonlyArray<number> = [1024, 2048, 4096];

export const RENDER_SHADOW_RESOLUTION_OPTIONS: ReadonlyArray<IRenderChoiceOption<string>> =
  RENDER_SHADOW_RESOLUTIONS.map((value: number) => ({ label: String(value), value: String(value) }));

/** The bounds each shadow value is offered between. */
export const RENDER_SHADOW_LIMITS = {
  bias: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.bias, 0.25),
  blend: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.blend, 0.01),
  filter: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.filter, 1),
  reach: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.reach, 50),
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
  return blend > 0 ? formatPercent(blend) : "Hard";
}

/** @returns A shadow's normal offset, in texels. */
export function formatShadowBias(bias: number): string {
  return formatNumber(bias, 2);
}

/** @returns How far towards the sun a cascade's casters may stand, in metres. */
export function formatShadowReach(reach: number): string {
  return `${formatNumber(reach, 0)} m`;
}

/** The bounds each contact shadow value is offered between. */
export const RENDER_CONTACT_SHADOW_LIMITS = {
  intensity: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.intensity, 0.05),
  length: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.length, 0.05),
  lights: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.lights, 1),
  steps: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.steps, 1),
  thickness: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.thickness, 0.01),
} as const;

/** @returns How far a contact shadow's ray reaches towards the sun or a light, in metres. */
export function formatContactShadowLength(length: number): string {
  return `${formatNumber(length, 2)} m`;
}

/** @returns How much light a contact shadow takes away. */
export function formatContactShadowIntensity(intensity: number): string {
  return formatPercent(intensity);
}

/** @returns How deep what the depth shows is taken to be, in centimetres. */
export function formatContactShadowThickness(thickness: number): string {
  return `${formatNumber(thickness * 100, 0)} cm`;
}

/** @returns How many depth reads a contact shadow's ray takes. */
export function formatContactShadowSteps(steps: number): string {
  return formatNumber(steps, 0);
}

/**
 * @param lights - Local lights each pixel marches contact shadows towards at most, the strongest there.
 * @returns It as the popover reads it: a count of lights, or the sun's alone for none.
 */
export function formatContactShadowLights(lights: number): string {
  return lights ? `${lights} light${lights > 1 ? "s" : ""}` : "Sun only";
}

/** The engine's grass density, `r__detail_density`, which is a spacing: a smaller one plants more. */
const GAME_GRASS_DENSITY: number = DEFAULT_RENDER_GRASS_SETTINGS.density;

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
 * The bounds each grass value is offered between, the settings' own: density as how many times the game's the tufts
 * stand, from its sparsest to three times it.
 */
export const RENDER_GRASS_LIMITS = {
  density: {
    max: toGrassDensityScale(RENDER_FEATURE_SCHEMA.grass.density.min),
    min: toGrassDensityScale(RENDER_FEATURE_SCHEMA.grass.density.max),
    step: 0.05,
  },
  height: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.height, 0.1),
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.grass.radius, 1),
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
export const RENDER_SHARPENING_LIMITS: IRenderLimits = toRenderLimits(RENDER_FEATURE_SCHEMA.upscaling.sharpening, 0.05);

/** @returns How much the upscaled frame is sharpened. */
export function formatSharpening(sharpening: number): string {
  return formatNumber(sharpening, 2);
}

/** The bounds each ambient occlusion value is offered between. */
export const RENDER_AMBIENT_OCCLUSION_LIMITS = {
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.radius, 0.25),
  strength: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.strength, 0.1),
} as const;

/** @returns An occlusion radius, in metres. */
export function formatOcclusionRadius(radius: number): string {
  return `${formatNumber(radius, 2)} m`;
}

/** The bounds each exposure value is offered between: the console's own. */
export const RENDER_EXPOSURE_LIMITS = {
  adaptation: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.adaptation, 0.01),
  amount: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.amount, 0.05),
  lowLuminance: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.lowLuminance, 0.0001),
  middleGray: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.middleGray, 0.05),
} as const;

/** @returns An exposure value that reads in hundredths: its middle gray, amount or adaptation speed. */
export function formatExposure(value: number): string {
  return formatNumber(value, 2);
}

/** @returns The luminance the measured one is floored at, which is set in ten-thousandths. */
export function formatLowLuminance(luminance: number): string {
  return formatNumber(luminance, 4);
}

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

/** @returns An occlusion strength. */
export function formatOcclusionStrength(strength: number): string {
  return formatNumber(strength, 1);
}

const RENDER_SCALE_NAMES: Readonly<Record<RenderScale, string>> = {
  [ERenderScale.NATIVE]: "Native",
  [ERenderScale.QUALITY]: "Quality",
  [ERenderScale.BALANCED]: "Balanced",
  [ERenderScale.PERFORMANCE]: "Performance",
};

/**
 * @param scale - A render scale.
 * @returns Its name as the settings say it, with the share of each side it draws.
 */
export function describeRenderScale(scale: RenderScale): string {
  return RENDER_SCALE_NAMES[scale];
}

/** The render scales, in the order they are offered. */
export const RENDER_SCALE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderScale>> = Object.values(ERenderScale).map(
  (value: RenderScale) => ({ label: describeRenderScale(value), value })
);

const RENDER_AMBIENT_OCCLUSION_QUALITY_NAMES: Readonly<Record<RenderAmbientOcclusionQuality, string>> = {
  [ERenderAmbientOcclusionQuality.LOW]: "Low",
  [ERenderAmbientOcclusionQuality.MEDIUM]: "Medium",
  [ERenderAmbientOcclusionQuality.HIGH]: "High",
  [ERenderAmbientOcclusionQuality.ULTRA]: "Ultra",
};

/**
 * @param quality - An ambient occlusion quality.
 * @returns Its name as the settings say it.
 */
export function describeRenderAmbientOcclusionQuality(quality: RenderAmbientOcclusionQuality): string {
  return RENDER_AMBIENT_OCCLUSION_QUALITY_NAMES[quality];
}

/** The ambient occlusion qualities, in the order they are offered. */
export const RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS: ReadonlyArray<
  IRenderChoiceOption<RenderAmbientOcclusionQuality>
> = Object.values(ERenderAmbientOcclusionQuality).map((value: RenderAmbientOcclusionQuality) => ({
  label: describeRenderAmbientOcclusionQuality(value),
  value,
}));

const RENDER_ANTIALIASING_NAMES: Readonly<Record<RenderAntialiasing, string>> = {
  [ERenderAntialiasing.NONE]: "None",
  [ERenderAntialiasing.FXAA]: "FXAA",
  [ERenderAntialiasing.SMAA]: "SMAA",
  [ERenderAntialiasing.TAA]: "TAA",
  [ERenderAntialiasing.FSR2]: "FSR 2",
};

/**
 * @param mode - An antialiasing mode.
 * @returns Its name as the settings say it.
 */
export function describeRenderAntialiasing(mode: RenderAntialiasing): string {
  return RENDER_ANTIALIASING_NAMES[mode];
}

/** Every antialiasing mode, in the order they are offered. */
export const RENDER_ANTIALIASING_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderAntialiasing>> = Object.values(
  ERenderAntialiasing
).map((value: RenderAntialiasing) => ({ label: describeRenderAntialiasing(value), value }));

const RENDER_LIGHT_SHADOW_FILTER_NAMES: Readonly<Record<RenderLightShadowFilter, string>> = {
  [ERenderLightShadowFilter.ENGINE]: "Engine",
  [ERenderLightShadowFilter.SOFT]: "Soft",
};

/**
 * @param filter - How a light's shadow is filtered.
 * @returns Its name as the settings say it.
 */
export function describeRenderLightShadowFilter(filter: RenderLightShadowFilter): string {
  return RENDER_LIGHT_SHADOW_FILTER_NAMES[filter];
}

/** The filters a light's shadow is offered in, in the order they are offered. */
export const RENDER_LIGHT_SHADOW_FILTER_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderLightShadowFilter>> =
  Object.values(ERenderLightShadowFilter).map((value: RenderLightShadowFilter) => ({
    label: describeRenderLightShadowFilter(value),
    value,
  }));

/** The frame rate limits, in the order they are offered. */
export const RENDER_FRAME_RATE_OPTIONS: ReadonlyArray<IRenderChoiceOption<TFrameRateLimit>> = FRAME_RATE_LIMITS.map(
  (value: TFrameRateLimit) => ({ label: value === "none" ? "No cap" : `${value} fps`, value })
);

/**
 * @param resolution - A resolution every viewport may draw at.
 * @returns Its name as the settings say it.
 */
export function describeRenderResolution(resolution: ERenderResolution): string {
  switch (resolution) {
    case ERenderResolution.WINDOW:
      return "Window";
    case ERenderResolution.HEIGHT_720:
      return "720p";
    case ERenderResolution.HEIGHT_1080:
      return "1080p";
    case ERenderResolution.HEIGHT_1440:
      return "1440p";
    case ERenderResolution.HEIGHT_2160:
      return "4K";
  }
}

/** The resolutions, in the order they are offered. */
export const RENDER_RESOLUTION_OPTIONS: ReadonlyArray<IRenderChoiceOption<ERenderResolution>> = RENDER_RESOLUTIONS.map(
  (value: ERenderResolution) => ({ label: describeRenderResolution(value), value })
);
