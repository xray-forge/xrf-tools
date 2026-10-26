import { formatNumber } from "@xrf/format";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The physical light's grading sliders' bounds and steps. */
export const RENDER_GRADING_LIMITS: Readonly<Record<"contrast" | "saturation", IRenderLimits>> = {
  contrast: toRenderLimits(RENDER_FEATURE_SCHEMA.grading.contrast, 0.01),
  saturation: toRenderLimits(RENDER_FEATURE_SCHEMA.grading.saturation, 0.01),
};

/** @returns A grading factor in hundredths. */
export function formatGrading(value: number): string {
  return formatNumber(value, 2);
}
