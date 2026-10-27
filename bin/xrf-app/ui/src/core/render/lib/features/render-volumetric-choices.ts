import { formatNumber } from "@xrf/format";
import { ERenderVolumetricMode, RenderVolumetricMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/ChoiceFormRow/choice-form-row-option";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each enhanced volumetric lighting value is offered between. */
export const RENDER_VOLUMETRIC_LIMITS = {
  anisotropy: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.anisotropy, 0.05),
  distance: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.distance, 5),
  intensity: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.intensity, 0.05),
} as const;

const RENDER_VOLUMETRIC_MODE_DESCRIPTIONS: Readonly<Record<RenderVolumetricMode, string>> = {
  [ERenderVolumetricMode.ENGINE]:
    "Draws the game's shafts: the sun's marched along each pixel's ray, the shadowed lamps' apart.",
  [ERenderVolumetricMode.ENHANCED]:
    "Scatters the sun and every lamp through their shadows in one volume along the view, in place of the shafts.",
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
