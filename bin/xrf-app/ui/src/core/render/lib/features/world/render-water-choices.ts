import { formatNumber, formatPercent } from "@xrf/format";
import { ERenderWaterMode, RenderWaterMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import {
  ERenderSettingControl,
  IRenderSliderSetting,
  TRenderSettings,
} from "@/core/render/lib/features/render-setting";
import {
  IRenderFeatureSettings,
  TRenderEnhancedWaterSettings,
  TRenderWaterSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

/** @returns A wave's height, in centimetres, which is the size the engine's are. */
export function formatWaveHeight(height: number): string {
  return `${formatNumber(height * 100, 1)} cm`;
}

/** @returns How fast the waves travel, in the engine's own units. */
export function formatWaveSpeed(speed: number): string {
  return formatNumber(speed, 0);
}

/** @returns A multiple of the engine's own value. */
export function formatWaterMultiple(multiple: number): string {
  return `${formatNumber(multiple, 2)}×`;
}

/** @returns One of the enhanced water's strengths. */
export function formatWaterStrength(strength: number): string {
  return formatNumber(strength, 2);
}

/** @returns The depth the enhanced water's edge fades over. */
export function formatWaterBorder(depth: number): string {
  return `${formatNumber(depth, 2)} m`;
}

/** @returns How far the distortion moves what is behind the water and distorting particles, a share of the screen. */
export function formatWaterDistortion(distortion: number): string {
  return `${formatNumber(distortion * 100, 1)}%`;
}

const RENDER_WATER_MODE_DESCRIPTIONS: Readonly<Record<RenderWaterMode, string>> = {
  [ERenderWaterMode.ENGINE]: "Draws the game's rippled water, reflecting the sky.",
  [ERenderWaterMode.ENHANCED]:
    "Refracts and clouds what lies under the water by its depth, reflects the scene and softens the shore.",
};

/**
 * @param mode - A water mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderWaterMode(mode: RenderWaterMode): string {
  return RENDER_WATER_MODE_DESCRIPTIONS[mode];
}

/** The water modes, in the order they are offered. */
export const RENDER_WATER_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderWaterMode>> = [
  { label: "Engine", value: ERenderWaterMode.ENGINE },
  { label: "Enhanced", value: ERenderWaterMode.ENHANCED },
];

/** Whether no water is drawn, which none of its values tune. */
function isUndrawnWater(water: TRenderWaterSettings): boolean {
  return !water.isEnabled;
}

/**
 * How each water setting is set, in the settings and the Water popover alike; both the engine's water and the
 * enhanced one draw them, the popover's sections placing each by what it shapes.
 */
export const RENDER_WATER_SETTINGS: TRenderSettings<TRenderWaterSettings> = {
  distortion: {
    control: ERenderSettingControl.SLIDER,
    entryLabel: "Strength",
    format: formatWaterDistortion,
    isInert: (water: TRenderWaterSettings) => isUndrawnWater(water) || !water.isDistorted,
    label: "Distortion strength",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.water.distortion, 0.005),
  },
  enhanced: { control: ERenderSettingControl.CUSTOM, label: "Enhanced water" },
  isDistorted: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Refracts what is seen through the surface.",
    isInert: isUndrawnWater,
    label: "Distortion",
  },
  isEnabled: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Ripples and sky reflection.",
    label: "Water",
  },
  isSoft: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Fades the water by its depth and lays foam in the shallows. Game setting: r2_soft_water.",
    isInert: isUndrawnWater,
    label: "Soft water",
  },
  mode: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderWaterMode,
    isEntryDescribed: true,
    label: "Mode",
    options: RENDER_WATER_MODE_OPTIONS,
  },
  reflection: {
    control: ERenderSettingControl.SLIDER,
    description: "How strongly the water reflects the sky's cube, against the game's.",
    entryLabel: "Sky",
    format: formatWaterMultiple,
    label: "Reflection",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.water.reflection, 0.05),
  },
  ripple: {
    control: ERenderSettingControl.SLIDER,
    format: formatWaterMultiple,
    label: "Ripple",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.water.ripple, 0.05),
  },
  waveHeight: {
    control: ERenderSettingControl.SLIDER,
    entryLabel: "Height",
    format: formatWaveHeight,
    label: "Wave height",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.water.waveHeight, 0.001),
  },
  waveSpeed: {
    control: ERenderSettingControl.SLIDER,
    entryLabel: "Speed",
    format: formatWaveSpeed,
    label: "Wave speed",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.water.waveSpeed, 1),
  },
};

/**
 * @param label - What it is called.
 * @param limits - Between what it is offered.
 * @param format - How it reads.
 * @returns One of the enhanced water's values, which tune nothing of the game's water.
 */
function toEnhancedWaterSlider(
  label: string,
  limits: IRenderLimits,
  format: (value: number) => string
): IRenderSliderSetting<TRenderEnhancedWaterSettings> {
  return {
    control: ERenderSettingControl.SLIDER,
    format,
    isInert: (_: TRenderEnhancedWaterSettings, features: IRenderFeatureSettings) =>
      features.water.mode !== ERenderWaterMode.ENHANCED,
    label,
    limits,
  };
}

/** How each of the enhanced water's values is set, in the Water popover. */
export const RENDER_ENHANCED_WATER_SETTINGS: TRenderSettings<TRenderEnhancedWaterSettings> = {
  blurNoise: toEnhancedWaterSlider(
    "Blur noise",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.blurNoise, 0.05),
    formatWaterStrength
  ),
  calmFlow: toEnhancedWaterSlider(
    "Calm flow",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.calmFlow, 0.05),
    formatPercent
  ),
  caustics: toEnhancedWaterSlider(
    "Caustics",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.caustics, 0.01),
    formatWaterStrength
  ),
  flow: toEnhancedWaterSlider(
    "Flow",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.flow, 0.05),
    formatWaterMultiple
  ),
  parallaxHeight: toEnhancedWaterSlider(
    "Parallax height",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.parallaxHeight, 0.005),
    formatWaveHeight
  ),
  reflectionBlur: {
    ...toEnhancedWaterSlider(
      "Reflection blur",
      toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.reflectionBlur, 0.05),
      formatWaterStrength
    ),
    entryLabel: "Blur",
  },
  reflectivity: toEnhancedWaterSlider(
    "Reflectivity",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.reflectivity, 0.05),
    formatWaterStrength
  ),
  refraction: toEnhancedWaterSlider(
    "Refraction",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.refraction, 0.05),
    formatWaterStrength
  ),
  ripples: toEnhancedWaterSlider(
    "Rain ripples",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.ripples, 0.05),
    formatWaterStrength
  ),
  softBorder: toEnhancedWaterSlider(
    "Soft border",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.softBorder, 0.05),
    formatWaterBorder
  ),
  specular: toEnhancedWaterSlider(
    "Specular",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.specular, 0.1),
    formatWaterStrength
  ),
  turbidity: toEnhancedWaterSlider(
    "Turbidity",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.turbidity, 0.1),
    formatWaterStrength
  ),
  variation: toEnhancedWaterSlider(
    "Variation",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.variation, 0.05),
    formatWaterStrength
  ),
  vibrance: toEnhancedWaterSlider(
    "Vibrance",
    toRenderLimits(RENDER_FEATURE_SCHEMA.water.enhanced.vibrance, 0.05),
    formatPercent
  ),
};
