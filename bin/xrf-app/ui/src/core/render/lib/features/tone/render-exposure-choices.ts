import { formatNumber } from "@xrf/format";
import { ERenderLightModel, ERenderTonemapMode, RenderTonemapMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import {
  ERenderSettingControl,
  IRenderSliderSetting,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import {
  IRenderFeatureSettings,
  TRenderExposureSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

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

/** Whether the light is physical, which meters by its histogram and takes no engine meter's value. */
function isPhysicalLight(_: TRenderExposureSettings, features: IRenderFeatureSettings): boolean {
  return features.lightModel === ERenderLightModel.PHYSICAL;
}

/**
 * @param label - What it is called.
 * @param description - What it does, and the console variable it is.
 * @param limits - Between what it is offered: the console's own bounds.
 * @param format - How it reads.
 * @returns One of the engine meter's values, read only while it adapts under the engine's light.
 */
function toMeterSlider(
  label: string,
  description: string,
  limits: IRenderLimits,
  format: (value: number) => string
): IRenderSliderSetting<TRenderExposureSettings> {
  return {
    control: ERenderSettingControl.SLIDER,
    description,
    format,
    isInert: (exposure: TRenderExposureSettings, features: IRenderFeatureSettings) =>
      !exposure.isEnabled || isPhysicalLight(exposure, features),
    label,
    limits,
  };
}

/** How each exposure setting is set, in the settings and the Look popover alike, the look's values edited by it. */
export const RENDER_EXPOSURE_SETTINGS: TRenderSettings<TRenderExposureSettings> = {
  adaptation: toMeterSlider(
    "Adaptation speed",
    "How quickly the exposure follows the frame. Game setting: r2_tonemap_adaptation.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.adaptation, 0.01),
    formatExposure
  ),
  amount: toMeterSlider(
    "Strength",
    "Share of the adaptation applied. Game setting: r2_tonemap_amount.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.amount, 0.05),
    formatExposure
  ),
  compensation: {
    control: ERenderSettingControl.SLIDER,
    description: "Stops the physical light's exposure lifts the frame over the level it meters it at, or lowers it.",
    format: formatStops,
    isInert: (exposure: TRenderExposureSettings, features: IRenderFeatureSettings) =>
      !isPhysicalLight(exposure, features),
    label: "Compensation",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.compensation, 0.1),
  },
  isEnabled: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Adapts brightness to the frame's average luminance. Off, the engine's fixed daylight exposure is used, or the " +
      "compensation under the physical light. Game setting: r2_tonemap.",
    isEntryDescribed: true,
    label: "Auto exposure",
  },
  lowLuminance: toMeterSlider(
    "Luminance floor",
    "Lowest luminance the meter reads. Game setting: r2_tonemap_lowlum.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.lowLuminance, 0.0001),
    formatLowLuminance
  ),
  middleGray: toMeterSlider(
    "Middle gray",
    "Target luminance. Game setting: r2_tonemap_middlegray.",
    toRenderLimits(RENDER_FEATURE_SCHEMA.exposure.middleGray, 0.05),
    formatExposure
  ),
};
