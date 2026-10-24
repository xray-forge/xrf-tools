import { formatNumber, formatPercent } from "@xrf/format";
import { ERenderFogMode, RenderFogMode } from "@xrf/ipc/types/xrf-renderer";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The bounds each enhanced fog value is offered between. */
export const RENDER_FOG_LIMITS = {
  density: toRenderLimits(RENDER_FEATURE_SCHEMA.fog.density, 0.1),
  height: toRenderLimits(RENDER_FEATURE_SCHEMA.fog.height, 0.5),
  skyFade: toRenderLimits(RENDER_FEATURE_SCHEMA.fog.skyFade, 0.05),
  scattering: toRenderLimits(RENDER_FEATURE_SCHEMA.fog.scattering, 0.05),
  sunColor: toRenderLimits(RENDER_FEATURE_SCHEMA.fog.sunColor, 0.05),
} as const;

const RENDER_FOG_MODE_DESCRIPTIONS: Readonly<Record<RenderFogMode, string>> = {
  [ERenderFogMode.ENGINE]: "Draws the weather's distance fog, as the game does.",
  [ERenderFogMode.ENHANCED]:
    "Thickens the distance fog low down, tints it towards the sun and scatters the frame's bright parts into it.",
};

/**
 * @param mode - A fog mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderFogMode(mode: RenderFogMode): string {
  return RENDER_FOG_MODE_DESCRIPTIONS[mode];
}

/** The fog modes, in the order they are offered. */
export const RENDER_FOG_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderFogMode>> = [
  { label: "Engine", value: ERenderFogMode.ENGINE },
  { label: "Enhanced", value: ERenderFogMode.ENHANCED },
];

/** @returns How far either side of the world's zero the height fog rises over. */
export function formatFogHeight(height: number): string {
  return `${formatNumber(height, 1)} m`;
}

/** @returns How much thicker the height fog makes the distance fog, as a share of it. */
export function formatFogDensity(density: number): string {
  return `+${formatPercent(density)}`;
}

/** @returns The fog's share past which fogged surfaces fade into the sky, or the engine's fade from the first. */
export function formatFogSkyFade(share: number): string {
  return share > 0 ? `${Math.round(share * 100)}% fog` : "Engine";
}

/** @returns A share of the enhanced fog's: its sun colour or its scattering. */
export function formatFogShare(share: number): string {
  return formatPercent(share);
}
