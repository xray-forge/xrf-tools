import { formatNumber, formatPercent } from "@xrf/format";

import { ERenderLightShadowFilter, RenderLightShadowFilter } from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_SHADOW_CASCADE_WIDTHS } from "@/core/render/lib/settings/render-feature-defaults";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** How many cascades are offered, as counts. */
export const RENDER_SHADOW_CASCADE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> =
  RENDER_SHADOW_CASCADE_WIDTHS.map((_, index: number) => ({ label: String(index + 1), value: String(index + 1) }));

/** The map resolutions offered, in texels across. */
const RENDER_SHADOW_RESOLUTIONS: ReadonlyArray<number> = [1024, 1536, 2048, 4096];

export const RENDER_SHADOW_RESOLUTION_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> =
  RENDER_SHADOW_RESOLUTIONS.map((value: number) => ({ label: String(value), value: String(value) }));

/** The bounds each shadow value is offered between. */
export const RENDER_SHADOW_LIMITS = {
  bias: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.bias, 0.25),
  blend: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.blend, 0.01),
  filter: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.filter, 1),
} as const;

/**
 * @param texels - Texels each way a shadow's edge is averaged over.
 * @returns It as both surfaces read it: a width, or the hardest edge for none.
 */
export function formatShadowFilter(texels: number): string {
  return texels ? `${texels} texel${texels > 1 ? "s" : ""}` : "Hard";
}

/**
 * @param blend - How far in from a cascade's edge the next is mixed in, as a share of its width.
 * @returns It as the settings read it: a percentage, or the engine's hard switch for none.
 */
export function formatCascadeBlend(blend: number): string {
  return blend > 0 ? formatPercent(blend) : "Hard";
}

/** @returns A shadow's normal offset, in texels. */
export function formatShadowBias(bias: number): string {
  return `${formatNumber(bias, 2)} texels`;
}

const RENDER_LIGHT_SHADOW_FILTER_NAMES: Readonly<Record<RenderLightShadowFilter, string>> = {
  [ERenderLightShadowFilter.ENGINE]: "Engine",
  [ERenderLightShadowFilter.SOFT]: "Soft",
};

/**
 * @param filter - How a light's shadow is filtered.
 * @returns Its name as the settings say it.
 */
export function describeRenderLightShadowFilter(filter: RenderLightShadowFilter): string {
  return RENDER_LIGHT_SHADOW_FILTER_NAMES[filter];
}

/** The filters a light's shadow is offered in, in the order they are offered. */
export const RENDER_LIGHT_SHADOW_FILTER_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightShadowFilter>> =
  Object.values(ERenderLightShadowFilter).map((value: RenderLightShadowFilter) => ({
    label: describeRenderLightShadowFilter(value),
    value,
  }));
