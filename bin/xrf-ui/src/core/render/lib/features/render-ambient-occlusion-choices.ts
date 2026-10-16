import { formatNumber, formatPercent } from "@xrf/format";

import {
  ERenderAmbientOcclusionMethod,
  ERenderAmbientOcclusionQuality,
  RenderAmbientOcclusionMethod,
  RenderAmbientOcclusionQuality,
} from "@/core/ipc/types/xrf-renderer";
import { IRenderChoiceOption } from "@/core/render/lib/features/render-choice-option";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each ambient occlusion value is offered between. */
export const RENDER_AMBIENT_OCCLUSION_LIMITS = {
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.radius, 0.25),
  strength: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.strength, 0.1),
} as const;

const RENDER_AMBIENT_OCCLUSION_METHOD_NAMES: Readonly<Record<RenderAmbientOcclusionMethod, string>> = {
  [ERenderAmbientOcclusionMethod.GTAO]: "GTAO",
  [ERenderAmbientOcclusionMethod.VBAO]: "VBAO",
  [ERenderAmbientOcclusionMethod.RTAO]: "RTAO",
};

const RENDER_AMBIENT_OCCLUSION_METHOD_DESCRIPTIONS: Readonly<Record<RenderAmbientOcclusionMethod, string>> = {
  [ERenderAmbientOcclusionMethod.GTAO]:
    "XeGTAO's horizons: whatever stands in front darkens all that lies behind it, however thin.",
  [ERenderAmbientOcclusionMethod.VBAO]:
    "A visibility bitmask: every occluder a thickness deep, so light passes behind grass, fences and branches.",
  [ERenderAmbientOcclusionMethod.RTAO]:
    "Rays traced against the level's geometry: what is off screen or behind a nearer surface occludes too. Grass " +
    "is not traced and occludes nothing.",
};

/**
 * @param method - An ambient occlusion method.
 * @returns Its name, as the popover offers it.
 */
export function describeRenderAmbientOcclusionMethod(method: RenderAmbientOcclusionMethod): string {
  return RENDER_AMBIENT_OCCLUSION_METHOD_NAMES[method];
}

/**
 * @param method - An ambient occlusion method.
 * @returns What it does, in a sentence.
 */
export function explainRenderAmbientOcclusionMethod(method: RenderAmbientOcclusionMethod): string {
  return RENDER_AMBIENT_OCCLUSION_METHOD_DESCRIPTIONS[method];
}

/** The ambient occlusion methods, in the order they are offered. */
export const RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderAmbientOcclusionMethod>> =
  Object.values(ERenderAmbientOcclusionMethod).map((value: RenderAmbientOcclusionMethod) => ({
    label: describeRenderAmbientOcclusionMethod(value),
    value,
  }));

/** The same where the GPU traces no rays: RTAO greyed out, VBAO searching in its place. */
export const RENDER_AMBIENT_OCCLUSION_METHOD_UNTRACED_OPTIONS: ReadonlyArray<
  IRenderChoiceOption<RenderAmbientOcclusionMethod>
> = RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS.map((option: IRenderChoiceOption<RenderAmbientOcclusionMethod>) =>
  option.value === ERenderAmbientOcclusionMethod.RTAO ? { ...option, isDisabled: true } : option
);

/** The bounds each VBAO value is offered between. */
export const RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS = {
  accumulation: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.accumulation, 1),
  bounce: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.bounce, 0.05),
  thickness: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.thickness, 0.01),
} as const;

/** @returns How deep an occluder is taken to be, in metres. */
export function formatOcclusionThickness(thickness: number): string {
  return `${formatNumber(thickness, 2)} m`;
}

/** @returns How much of the light bounced between a crease's sides comes back. */
export function formatOcclusionBounce(bounce: number): string {
  return formatPercent(bounce);
}

/** @returns Over how many frames the occlusion is averaged, or that it is not. */
export function formatOcclusionAccumulation(accumulation: number): string {
  return accumulation > 1 ? `${accumulation} frames` : "None";
}

/** @returns An occlusion radius, in metres. */
export function formatOcclusionRadius(radius: number): string {
  return `${formatNumber(radius, 2)} m`;
}

/** @returns An occlusion strength. */
export function formatOcclusionStrength(strength: number): string {
  return formatNumber(strength, 1);
}

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
