import { formatCount, formatNumber } from "@xrf/format";
import { ERenderDebandingMode, RenderDebandingMode } from "@xrf/ipc/types/xrf-renderer";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderDebandingSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

const RENDER_DEBANDING_MODE_DESCRIPTIONS: Readonly<Record<RenderDebandingMode, string>> = {
  [ERenderDebandingMode.ENGINE]: "Draws the sky as the game does.",
  [ERenderDebandingMode.ENHANCED]:
    "Smooths the sky's colour bands, averaging each sky pixel with neighbours less than a band apart.",
};

/**
 * @param mode - A sky debanding mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderDebandingMode(mode: RenderDebandingMode): string {
  return RENDER_DEBANDING_MODE_DESCRIPTIONS[mode];
}

/** @returns How many times a sky pixel is averaged with its neighbours. */
export function formatDebandingPasses(passes: number): string {
  return passes === 1 ? "Once" : `${formatCount(passes)} times`;
}

/** @returns How far out a sky pixel's neighbours are read at most. */
export function formatDebandingRadius(radius: number): string {
  return `${formatNumber(radius, 0)} px`;
}

/** Whether the sky is drawn as the game draws it, which none of the debanding's values tune. */
function isEngineDebanding(debanding: TRenderDebandingSettings): boolean {
  return debanding.mode === ERenderDebandingMode.ENGINE;
}

/** How each sky debanding setting is set, in the settings and the Look popover alike. */
export const RENDER_DEBANDING_SETTINGS: TRenderSettings<TRenderDebandingSettings> = {
  // Set as a toggle for the enhanced mode, which says what the mode drawn does.
  mode: {
    control: ERenderSettingControl.CUSTOM,
    label: "Debanding",
  },
  passes: {
    control: ERenderSettingControl.SLIDER,
    description: "Times a sky pixel is averaged with its neighbours, each time further out.",
    entryLabel: "Debanding passes",
    format: formatDebandingPasses,
    isInert: isEngineDebanding,
    label: "Passes",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.debanding.passes, 1),
  },
  radius: {
    control: ERenderSettingControl.SLIDER,
    description: "Pixels around a sky pixel its neighbours are read from at most.",
    entryLabel: "Debanding radius",
    format: formatDebandingRadius,
    isInert: isEngineDebanding,
    label: "Radius",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.debanding.radius, 4),
  },
};
