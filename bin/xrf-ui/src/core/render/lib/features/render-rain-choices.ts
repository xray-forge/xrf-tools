import { formatPercent } from "@xrf/format";
import {
  ERenderRainDropsMode,
  ERenderRainMode,
  RenderRainDropsMode,
  RenderRainMode,
} from "@xrf/ipc/types/xrf-renderer";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each rain value is offered between. */
export const RENDER_RAIN_LIMITS = {
  distance: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.distance, 10),
  dropSquare: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.dropSquare, 1),
  pace: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.pace, 1),
  puddles: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.puddles, 0.05),
  reflectivity: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.reflectivity, 0.05),
  ringDistance: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ringDistance, 5),
  ringSize: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ringSize, 0.05),
  ringStrength: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ringStrength, 0.05),
  ripples: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ripples, 0.05),
} as const;

const RENDER_RAIN_MODE_DESCRIPTIONS: Readonly<Record<RenderRainMode, string>> = {
  [ERenderRainMode.ENGINE]: "Wets surfaces as the game does: splashes and running water near the camera.",
  [ERenderRainMode.ENHANCED]:
    "Gathers puddles on flat, low ground over the game's wetting, filling with the rain and draining after.",
};

const RENDER_RAIN_DROPS_MODE_DESCRIPTIONS: Readonly<Record<RenderRainDropsMode, string>> = {
  [ERenderRainDropsMode.ENGINE]: "Draws the game's rain streaks, tinted by the weather's rain colour.",
  [ERenderRainDropsMode.ENHANCED]:
    "Draws thin drops that bend the scene behind them and catch its light, and splashes that bend the ground.",
};

/**
 * @param mode - A rain mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderRainMode(mode: RenderRainMode): string {
  return RENDER_RAIN_MODE_DESCRIPTIONS[mode];
}

/**
 * @param mode - A falling rain mode.
 * @returns What it draws, in a sentence.
 */
export function explainRenderRainDropsMode(mode: RenderRainDropsMode): string {
  return RENDER_RAIN_DROPS_MODE_DESCRIPTIONS[mode];
}

/** @returns A rain strength as a share. */
export function formatRainShare(share: number): string {
  return formatPercent(share);
}

/** @returns The pace the level wets at, as a multiple of the game's own. */
export function formatRainPace(pace: number): string {
  return `×${Math.round(pace)}`;
}
