import { formatNumber } from "@xrf/format";
import { ERenderLightModel } from "@xrf/ipc/types/xrf-renderer";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import {
  IRenderFeatureSettings,
  TRenderGradingSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

/** @returns A grading factor in hundredths. */
export function formatGrading(value: number): string {
  return formatNumber(value, 2);
}

/** Whether the light is the engine's, whose frame is graded by the look's corrections in place of these. */
function isEngineLight(_: TRenderGradingSettings, features: IRenderFeatureSettings): boolean {
  return features.lightModel !== ERenderLightModel.PHYSICAL;
}

/** How the physical light's grading is set, in the settings and the Look popover alike. */
export const RENDER_GRADING_SETTINGS: TRenderSettings<TRenderGradingSettings> = {
  contrast: {
    control: ERenderSettingControl.SLIDER,
    description: "Stops of the physical light's frame about middle grey, raised before its display transform.",
    entryLabel: "Contrast",
    format: formatGrading,
    isInert: isEngineLight,
    label: "Display contrast",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.grading.contrast, 0.01),
  },
  saturation: {
    control: ERenderSettingControl.SLIDER,
    description: "How far the physical light's frame is taken from grey before its display transform.",
    entryLabel: "Saturation",
    format: formatGrading,
    isInert: isEngineLight,
    label: "Display saturation",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.grading.saturation, 0.01),
  },
};
