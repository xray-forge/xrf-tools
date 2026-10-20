import { formatNumber } from "@xrf/format";

import { ERenderTonemapMode, RenderTonemapMode } from "@/core/ipc/types/xrf-renderer";
import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

/** The bounds each exposure value is offered between: the console's own. */
export const RENDER_EXPOSURE_LIMITS = {
  adaptation: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.adaptation, 0.01),
  amount: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.amount, 0.05),
  compensation: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.compensation, 0.1),
  lowLuminance: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.lowLuminance, 0.0001),
  middleGray: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.middleGray, 0.05),
} as const;

/** @returns Stops of exposure compensation, signed, in tenths. */
export function formatStops(stops: number): string {
  return `${stops > 0 ? "+" : ""}${formatNumber(stops, 1)} EV`;
}

/** @returns An exposure value that reads in hundredths: its middle gray, amount or adaptation speed. */
export function formatExposure(value: number): string {
  return formatNumber(value, 2);
}

/** @returns The luminance the measured one is floored at, which is set in ten-thousandths. */
export function formatLowLuminance(luminance: number): string {
  return formatNumber(luminance, 4);
}

const RENDER_TONEMAP_MODE_DESCRIPTIONS: Readonly<Record<RenderTonemapMode, string>> = {
  [ERenderTonemapMode.ENGINE]: "Tonemaps what each pass draws, as the game does.",
  [ERenderTonemapMode.ENHANCED]:
    "Keeps the frame in linear light until it is shown, so blends mix as light does and upscalers get their input.",
};

/**
 * @param mode - A tonemap mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderTonemapMode(mode: RenderTonemapMode): string {
  return RENDER_TONEMAP_MODE_DESCRIPTIONS[mode];
}

/** The tonemap's places, in the order they are offered. */
export const RENDER_TONEMAP_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderTonemapMode>> = [
  { label: "Engine", value: ERenderTonemapMode.ENGINE },
  { label: "Enhanced", value: ERenderTonemapMode.ENHANCED },
];
