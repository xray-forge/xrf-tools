import { formatNumber } from "@xrf/format";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import {
  ERenderSettingControl,
  IRenderSliderSetting,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import { TRenderCullingSettings, TRenderLodSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

/** How each culling setting is set, in the settings and the Culling popover alike. */
export const RENDER_CULLING_SETTINGS: TRenderSettings<TRenderCullingSettings> = {
  isDepthPrepassed: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Draws the opaque surfaces' depth first, so each pixel is shaded once where surfaces stack; costs a little over " +
      "open ground seen from above.",
    label: "Opaque depth pass",
  },
  isOcclusionCulled: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Skips static geometry hidden by nearer surfaces, tested in two phases against depth.",
    label: "Occlusion culling",
  },
};

/** @returns The scale every threshold takes, in hundredths. */
export function formatLodScale(scale: number): string {
  return formatNumber(scale, 2);
}

/** @returns A screen area threshold, whole. */
export function formatLodArea(area: number): string {
  return formatNumber(area, 0);
}

/** @returns The screen area an instanced place is discarded below, in tenths. */
export function formatLodDiscard(area: number): string {
  return formatNumber(area, 1);
}

/**
 * @param label - What it is called.
 * @param description - What it does, and the console variable it is.
 * @param limits - Between what it is offered: the renderer's schema's.
 * @param format - How it reads.
 * @returns One of the engine's thresholds by screen coverage.
 */
function toLodSlider(
  label: string,
  description: string,
  limits: IRenderLimits,
  format: (value: number) => string
): IRenderSliderSetting<TRenderLodSettings> {
  return { control: ERenderSettingControl.SLIDER, description, format, label, limits };
}

/**
 * How each level of detail setting is set, in the settings; the Culling popover sets impostors. The thresholds are
 * the engine console's (`xrRender_console.cpp`), in its order.
 */
export const RENDER_LOD_SETTINGS: TRenderSettings<TRenderLodSettings> = {
  geometryLod: toLodSlider(
    "Detail scale",
    "Scales every threshold below. Game setting: r__geometry_lod.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.lod.geometryLod, 0.05),
    formatLodScale
  ),
  isImpostors: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Draws distant tree clusters as impostors. Off, every tree renders at full detail.",
    label: "Impostors",
  },
  ssaA: toLodSlider(
    "Impostor below",
    "Clusters smaller on screen draw as impostors. Game setting: r2_ssa_lod_a.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.lod.ssaA, 1),
    formatLodArea
  ),
  ssaB: toLodSlider(
    "Trees above",
    "Clusters larger draw as trees; in between, both. Game setting: r2_ssa_lod_b.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.lod.ssaB, 1),
    formatLodArea
  ),
  ssaDiscard: toLodSlider(
    "Cull below",
    "Clusters smaller are not drawn. Game setting: r__ssa_discard.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.lod.ssaDiscard, 0.5),
    formatLodDiscard
  ),
  ssaGlodEnd: toLodSlider(
    "Lowest detail below",
    "Progressive trees smaller draw at the lowest detail. Game setting: r__ssa_glod_end.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.lod.ssaGlodEnd, 1),
    formatLodArea
  ),
  ssaGlodStart: toLodSlider(
    "Full detail above",
    "Progressive trees larger draw at full detail. Game setting: r__ssa_glod_start.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.lod.ssaGlodStart, 8),
    formatLodArea
  ),
};
