import { formatNumber } from "@xrf/format";

import { ERenderTonemapMode, RenderTonemapMode } from "@/core/ipc/types/xrf-renderer";
import { IRenderChoiceOption } from "@/core/render/lib/features/render-choice-option";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each exposure value is offered between: the console's own. */
export const RENDER_EXPOSURE_LIMITS = {
  adaptation: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.adaptation, 0.01),
  amount: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.amount, 0.05),
  lowLuminance: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.lowLuminance, 0.0001),
  middleGray: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.middleGray, 0.05),
} as const;

/** @returns An exposure value that reads in hundredths: its middle gray, amount or adaptation speed. */
export function formatExposure(value: number): string {
  return formatNumber(value, 2);
}

/** @returns The luminance the measured one is floored at, which is set in ten-thousandths. */
export function formatLowLuminance(luminance: number): string {
  return formatNumber(luminance, 4);
}

/** The tonemap's places, in the order they are offered. */
export const RENDER_TONEMAP_MODE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderTonemapMode>> = [
  { label: "Engine", value: ERenderTonemapMode.ENGINE },
  { label: "Enhanced", value: ERenderTonemapMode.ENHANCED },
];
