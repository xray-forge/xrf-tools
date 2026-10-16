import { formatPercent } from "@xrf/format";

import { ERenderRainMode, RenderRainMode } from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each rain value is offered between. */
export const RENDER_RAIN_LIMITS = {
  distance: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.distance, 10),
  puddles: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.puddles, 0.05),
  reflectivity: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.reflectivity, 0.05),
  ripples: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ripples, 0.05),
} as const;

const RENDER_RAIN_MODE_DESCRIPTIONS: Readonly<Record<RenderRainMode, string>> = {
  [ERenderRainMode.ENGINE]: "The game's own wetting alone: splashes and running water near the camera.",
  [ERenderRainMode.ENHANCED]:
    "Puddles gather on flat, low ground over the game's own wetting, filling with the rain and draining after.",
};

/**
 * @param mode - A rain mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderRainMode(mode: RenderRainMode): string {
  return RENDER_RAIN_MODE_DESCRIPTIONS[mode];
}

/** @returns A rain strength as a share. */
export function formatRainShare(share: number): string {
  return formatPercent(share);
}
