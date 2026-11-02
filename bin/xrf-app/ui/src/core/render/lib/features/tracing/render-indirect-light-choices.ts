import { formatCount, formatPercent } from "@xrf/format";
import { ERenderIndirectLightMode, RenderIndirectLightMode } from "@xrf/ipc/types/xrf-renderer";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { formatOcclusionRadius } from "@/core/render/lib/features/tracing/render-ambient-occlusion-choices";
import { TRenderIndirectLightSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

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

/** Whether the bounced light is the game's hemisphere and ambient, which none of the gathering's values tune. */
function isEngineIndirectLight(indirectLight: TRenderIndirectLightSettings): boolean {
  return indirectLight.mode === ERenderIndirectLightMode.ENGINE;
}

/** How each indirect light setting is set, in the settings and the Occlusion popover alike. */
export const RENDER_INDIRECT_LIGHT_SETTINGS: TRenderSettings<TRenderIndirectLightSettings> = {
  intensity: {
    control: ERenderSettingControl.SLIDER,
    description: "How much of the bounced light is added: 1 is all of it.",
    format: formatIndirectLightIntensity,
    isInert: isEngineIndirectLight,
    label: "Intensity",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.indirectLight.intensity, 0.05),
  },
  // Set as a toggle for the enhanced mode, which says what the mode drawn does.
  mode: {
    control: ERenderSettingControl.CUSTOM,
    label: "Indirect light",
  },
  radius: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres around a point the bounced light is gathered from.",
    format: formatOcclusionRadius,
    isInert: isEngineIndirectLight,
    label: "Bounce radius",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.indirectLight.radius, 0.25),
  },
  steps: {
    control: ERenderSettingControl.SLIDER,
    description: "Steps the search takes past the occlusion's radius out to the bounce radius.",
    format: formatCount,
    isInert: isEngineIndirectLight,
    label: "Bounce steps",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.indirectLight.steps, 1),
  },
};
