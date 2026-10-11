import { formatNumber } from "@xrf/format";

import { ERenderVolumetricMode, RenderVolumetricMode } from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The bounds each enhanced volumetric lighting value is offered between. */
export const RENDER_VOLUMETRIC_LIMITS = {
  anisotropy: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.anisotropy, 0.05),
  distance: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.distance, 5),
  intensity: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.intensity, 0.05),
} as const;

const RENDER_VOLUMETRIC_MODE_DESCRIPTIONS: Readonly<Record<RenderVolumetricMode, string>> = {
  [ERenderVolumetricMode.ENGINE]:
    "The game's own: the sun's shafts marched along each pixel's ray, and the shadowed lamps' shafts drawn apart.",
  [ERenderVolumetricMode.ENHANCED]:
    "The sun and every lamp scattered in one volume along the view, through their shadows, kept over frames, in the shafts' place.",
};

/**
 * @param mode - A volumetric lighting mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderVolumetricMode(mode: RenderVolumetricMode): string {
  return RENDER_VOLUMETRIC_MODE_DESCRIPTIONS[mode];
}

/** The volumetric lighting modes, in the order they are offered. */
export const RENDER_VOLUMETRIC_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderVolumetricMode>> = [
  { label: "Engine", value: ERenderVolumetricMode.ENGINE },
  { label: "Enhanced", value: ERenderVolumetricMode.ENHANCED },
];

/** @returns One of the enhanced volumetric lighting's strengths: its intensity or its anisotropy. */
export function formatVolumetricStrength(strength: number): string {
  return formatNumber(strength, 2);
}

/** @returns How far along the view the enhanced volumetric lighting reaches. */
export function formatVolumetricDistance(distance: number): string {
  return `${formatNumber(distance, 0)} m`;
}
