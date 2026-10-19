import { formatNumber } from "@xrf/format";

import { ERenderDebandingMode, RenderDebandingMode } from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds the sky debanding's radius is offered between. */
export const RENDER_DEBANDING_LIMITS = {
  passes: toRenderLimits(RENDER_FEATURE_SCHEMA.debanding.passes, 1),
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.debanding.radius, 4),
} as const;

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
  return passes === 1 ? "Once" : `${formatNumber(passes, 0)} times`;
}

/** @returns How far out a sky pixel's neighbours are read at most. */
export function formatDebandingRadius(radius: number): string {
  return `${formatNumber(radius, 0)} px`;
}
