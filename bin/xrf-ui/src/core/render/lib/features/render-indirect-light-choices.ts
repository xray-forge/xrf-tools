import { formatPercent } from "@xrf/format";

import { ERenderIndirectLightMode, RenderIndirectLightMode } from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each indirect light value is offered between. */
export const RENDER_INDIRECT_LIGHT_LIMITS = {
  intensity: toRenderLimits(RENDER_FEATURE_SCHEMA.indirectLight.intensity, 0.05),
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.indirectLight.radius, 0.25),
  steps: toRenderLimits(RENDER_FEATURE_SCHEMA.indirectLight.steps, 1),
} as const;

const RENDER_INDIRECT_LIGHT_MODE_DESCRIPTIONS: Readonly<Record<RenderIndirectLightMode, string>> = {
  [ERenderIndirectLightMode.ENGINE]: "Lets the hemisphere and the ambient stand for bounced light, as the game does.",
  [ERenderIndirectLightMode.ENHANCED]:
    "Bounces the sunlight and lamplight on what the frame shows once onto what faces it.",
};

/**
 * @param mode - An indirect light mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderIndirectLightMode(mode: RenderIndirectLightMode): string {
  return RENDER_INDIRECT_LIGHT_MODE_DESCRIPTIONS[mode];
}

/** @returns How much of the bounced light is added. */
export function formatIndirectLightIntensity(intensity: number): string {
  return formatPercent(intensity);
}
