import { formatNumber } from "@xrf/format";
import { ERenderVolumetricMode, RenderVolumetricMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderVolumetricSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

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

/** Whether the volume is the game's shafts, which none of the enhanced values tune. */
function isEngineVolumetric(volumetric: TRenderVolumetricSettings): boolean {
  return volumetric.mode === ERenderVolumetricMode.ENGINE;
}

/** How each volumetric lighting setting is set, in the settings and the Lamps popover alike. */
export const RENDER_VOLUMETRIC_SETTINGS: TRenderSettings<TRenderVolumetricSettings> = {
  anisotropy: {
    control: ERenderSettingControl.SLIDER,
    description: "How far scattered light keeps the way it was going: 0 scatters it every way alike.",
    format: formatVolumetricStrength,
    isInert: isEngineVolumetric,
    label: "Anisotropy",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.anisotropy, 0.05),
  },
  distance: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres along the view the volume reaches.",
    format: formatVolumetricDistance,
    isInert: isEngineVolumetric,
    label: "Distance",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.distance, 5),
  },
  intensity: {
    control: ERenderSettingControl.SLIDER,
    description: "How strongly the volume scatters: 1 as designed.",
    format: formatVolumetricStrength,
    isInert: isEngineVolumetric,
    label: "Intensity",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.volumetric.intensity, 0.05),
  },
  mode: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderVolumetricMode,
    label: "Mode",
    options: RENDER_VOLUMETRIC_MODE_OPTIONS,
  },
};
