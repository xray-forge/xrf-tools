import { formatNumber, formatPercent } from "@xrf/format";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { RENDER_SHADOW_CASCADE_WIDTHS } from "@/core/render/lib/settings/choice/render-feature-defaults";
import { TRenderShadowSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

/** How many cascades are offered, as counts. */
export const RENDER_SHADOW_CASCADE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> =
  RENDER_SHADOW_CASCADE_WIDTHS.map((_, index: number) => ({ label: String(index + 1), value: String(index + 1) }));

/**
 * @param widths - Cascade widths in metres.
 * @returns Them as the settings list them, the last joined by "and".
 */
export function formatCascadeWidths(widths: ReadonlyArray<number>): string {
  return `${widths.slice(0, -1).join(", ")} and ${widths.at(-1)} m`;
}

/** The map resolutions offered, in texels across. */
const RENDER_SHADOW_RESOLUTIONS: ReadonlyArray<number> = [1024, 1536, 2048, 4096];

export const RENDER_SHADOW_RESOLUTION_OPTIONS: ReadonlyArray<IChoiceFormRowOption<string>> =
  RENDER_SHADOW_RESOLUTIONS.map((value: number) => ({ label: String(value), value: String(value) }));

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

/** @returns How many of the nearest cascades redraw their swaying trees. */
export function formatShadowSwayCascades(count: number): string {
  return count === 0 ? "None" : formatNumber(count, 0);
}

/** @returns How many times a second the cascades redraw their swaying trees at most. */
export function formatShadowSwayRate(rate: number): string {
  return `${formatNumber(rate, 0)} Hz`;
}

/**
 * How each sun shadow setting is set, in the settings and the Sun shadows popover alike. The cascades are offered as a
 * count standing for their widths and the resolution as a few sizes, each a control of its own.
 */
export const RENDER_SHADOW_SETTINGS: TRenderSettings<TRenderShadowSettings> = {
  bias: {
    control: ERenderSettingControl.SLIDER,
    description: "Offset along the surface normal, in texels. Prevents self-shadowing.",
    format: formatShadowBias,
    label: "Normal offset",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.bias, 0.25),
  },
  blend: {
    control: ERenderSettingControl.SLIDER,
    description:
      "Transition band between cascades, as a share of cascade width. 0 switches abruptly, as the game does.",
    format: formatCascadeBlend,
    label: "Cascade blend",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.blend, 0.01),
  },
  cascades: {
    control: ERenderSettingControl.CUSTOM,
    description: `Cascade count. Widths: ${formatCascadeWidths(RENDER_SHADOW_CASCADE_WIDTHS)}.`,
    label: "Cascades",
  },
  contact: { control: ERenderSettingControl.CUSTOM, label: "Contact shadows" },
  filter: {
    control: ERenderSettingControl.SLIDER,
    description: "PCF kernel radius in texels. 0 gives hard edges.",
    format: formatShadowFilter,
    label: "Filter",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.filter, 1),
  },
  isEnabled: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Off, surfaces facing the sun are shaded by baked lighting alone.",
    label: "Sun shadows",
  },
  isStaggered: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Updates far cascades every second and fourth frame. Exact for static scenes; keeps camera motion smooth.",
    label: "Staggered updates",
  },
  resolution: {
    control: ERenderSettingControl.CUSTOM,
    description: "Texels per cascade map. Higher is sharper and uses more memory.",
    label: "Map resolution (texels)",
  },
  swayCascades: {
    control: ERenderSettingControl.SLIDER,
    description: "How many of the nearest cascades show trees swaying. Fewer is cheaper; far shadows hold still.",
    format: formatShadowSwayCascades,
    label: "Swaying cascades",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.swayCascades, 1),
  },
  swayRate: {
    control: ERenderSettingControl.SLIDER,
    description: "How often a second the cascades redraw swaying trees. Fewer is cheaper; their shadows step.",
    format: formatShadowSwayRate,
    label: "Sway updates",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.swayRate, 5),
  },
};
