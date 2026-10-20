import { formatCount, formatNumber, formatPercent } from "@xrf/format";

import {
  ERenderReflectionMode,
  ERenderReflectionProbe,
  RenderReflectionMode,
  RenderReflectionProbe,
} from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The bounds each reflection value is offered between. */
export const RENDER_REFLECTION_LIMITS = {
  distance: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.distance, 5),
  intensity: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.intensity, 0.05),
  reach: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.reach, 25),
  skyGain: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.skyGain, 0.05),
  steps: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.steps, 8),
  traceInterval: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.traceInterval, 1),
  traceRoughness: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.traceRoughness, 0.05),
  waterSteps: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.waterSteps, 4),
} as const;

const RENDER_REFLECTION_MODE_DESCRIPTIONS: Readonly<Record<RenderReflectionMode, string>> = {
  [ERenderReflectionMode.ENGINE]:
    "The ground, puddles and glossy surfaces reflect the sky's cube alone, as the game does",
  [ERenderReflectionMode.ENHANCED]:
    "The ground, puddles and glossy surfaces reflect what their rays meet on the screen, the sky's cube elsewhere",
};

/**
 * @param mode - A reflection mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderReflectionMode(mode: RenderReflectionMode): string {
  return RENDER_REFLECTION_MODE_DESCRIPTIONS[mode];
}

const RENDER_REFLECTION_PROBE_NAMES: Readonly<Record<RenderReflectionProbe, string>> = {
  [ERenderReflectionProbe.COARSE]: "256 texels",
  [ERenderReflectionProbe.FINE]: "512 texels",
  [ERenderReflectionProbe.FINEST]: "1024 texels",
};

/** The surroundings cube sizes, coarsest first. */
export const RENDER_REFLECTION_PROBE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderReflectionProbe>> =
  Object.values(ERenderReflectionProbe).map((value: RenderReflectionProbe) => ({
    label: RENDER_REFLECTION_PROBE_NAMES[value],
    value,
  }));

/** @returns What a surface's share of reflection is scaled by. */
export function formatReflectionIntensity(intensity: number): string {
  return formatPercent(intensity);
}

/** @returns How far a ray is traced. */
export function formatReflectionDistance(distance: number): string {
  return `${formatNumber(distance, 0)} m`;
}

/** @returns How far the sky's highlights are expanded as their light. */
export function formatReflectionSkyGain(gain: number): string {
  return `${formatNumber(gain, 2)}×`;
}

/** @returns How often a pixel the screen misses is traced again. */
export function formatReflectionTraceInterval(interval: number): string {
  return interval <= 1 ? "Every frame" : `Every ${formatCount(interval)} frames`;
}

/** @returns The roughness past which a surface is not traced. */
export function formatReflectionTraceRoughness(roughness: number): string {
  return formatNumber(roughness, 2);
}
