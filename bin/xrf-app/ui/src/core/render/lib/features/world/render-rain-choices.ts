import { formatPercent } from "@xrf/format";
import {
  ERenderRainDropsMode,
  ERenderRainMode,
  RenderRainDropsMode,
  RenderRainMode,
} from "@xrf/ipc/types/xrf-renderer";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { formatGrassRadius } from "@/core/render/lib/features/world/render-grass-choices";
import { TRenderRainSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

const RENDER_RAIN_MODE_DESCRIPTIONS: Readonly<Record<RenderRainMode, string>> = {
  [ERenderRainMode.ENGINE]: "Wets surfaces as the game does: splashes and running water near the camera.",
  [ERenderRainMode.ENHANCED]:
    "Gathers puddles on flat, low ground over the game's wetting, filling with the rain and draining after.",
};

const RENDER_RAIN_DROPS_MODE_DESCRIPTIONS: Readonly<Record<RenderRainDropsMode, string>> = {
  [ERenderRainDropsMode.ENGINE]: "Draws the game's rain streaks, tinted by the weather's rain colour.",
  [ERenderRainDropsMode.ENHANCED]:
    "Draws thin drops that bend the scene behind them and catch its light, and splashes that bend the ground.",
};

/**
 * @param mode - A rain mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderRainMode(mode: RenderRainMode): string {
  return RENDER_RAIN_MODE_DESCRIPTIONS[mode];
}

/**
 * @param mode - A falling rain mode.
 * @returns What it draws, in a sentence.
 */
export function explainRenderRainDropsMode(mode: RenderRainDropsMode): string {
  return RENDER_RAIN_DROPS_MODE_DESCRIPTIONS[mode];
}

/** @returns A rain strength as a share. */
export function formatRainShare(share: number): string {
  return formatPercent(share);
}

/** @returns The pace the level wets at, as a multiple of the game's own. */
export function formatRainPace(pace: number): string {
  return `×${Math.round(pace)}`;
}

/** Whether the rain falls as the game's streaks, which none of the refracting drops' values tune. */
function isEngineRain(rain: TRenderRainSettings): boolean {
  return rain.drops !== ERenderRainDropsMode.ENHANCED;
}

/** Whether no puddles gather, which none of the puddles' values tune. */
function isUnpuddled(rain: TRenderRainSettings): boolean {
  return rain.mode !== ERenderRainMode.ENHANCED;
}

/** How each rain setting is set, in the settings (Rain and Puddles) and the Rain popover alike. */
export const RENDER_RAIN_SETTINGS: TRenderSettings<TRenderRainSettings> = {
  distance: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres out to which puddles are drawn, fading over the last sixth.",
    format: formatGrassRadius,
    isInert: isUnpuddled,
    label: "Distance",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.distance, 10),
  },
  dropSquare: {
    control: ERenderSettingControl.SLIDER,
    description:
      "Metres across the square about the camera raindrops fall over; past 15 m they thin out with distance.",
    format: formatGrassRadius,
    isInert: isEngineRain,
    label: "Drop area",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.dropSquare, 1),
  },
  // Set as a toggle for the refracting drops, which says what the drops drawn do.
  drops: {
    control: ERenderSettingControl.CUSTOM,
    label: "Refracting drops",
  },
  isReliefFlooded: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Water fills the ground's hollows first where the enhanced parallax marches it.",
    isEntryDescribed: true,
    isInert: isUnpuddled,
    label: "Fill the relief",
  },
  // Set as the puddles' toggle, which says what the mode drawn does.
  mode: {
    control: ERenderSettingControl.CUSTOM,
    label: "Puddles",
  },
  pace: {
    control: ERenderSettingControl.SLIDER,
    description: "How many times the game's own pace the level wets, dries, and fills and drains its puddles at.",
    format: formatRainPace,
    label: "Wetting pace",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.pace, 1),
  },
  puddles: {
    control: ERenderSettingControl.SLIDER,
    description: "How many of the level's puddle sites hold water: about seven in ten at 100%.",
    format: formatRainShare,
    isInert: isUnpuddled,
    label: "Amount",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.puddles, 0.05),
  },
  reflectivity: {
    control: ERenderSettingControl.SLIDER,
    description: "How much of a reflection a puddle takes at most.",
    format: formatRainShare,
    isInert: isUnpuddled,
    label: "Reflectivity",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.reflectivity, 0.05),
  },
  ringDistance: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres out to which raindrops leave rings on water and puddles, with puddles enhanced.",
    format: formatGrassRadius,
    isInert: isEngineRain,
    label: "Ring distance",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ringDistance, 5),
  },
  ringSize: {
    control: ERenderSettingControl.SLIDER,
    description: "How wide raindrops' rings spread and how long their waves are, of their designed size.",
    format: formatRainShare,
    isInert: isEngineRain,
    label: "Ring size",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ringSize, 0.05),
  },
  ringStrength: {
    control: ERenderSettingControl.SLIDER,
    description: "How strongly raindrops' rings ripple water and puddles, of their designed strength.",
    format: formatRainShare,
    isInert: isEngineRain,
    label: "Ring strength",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ringStrength, 0.05),
  },
  ripples: {
    control: ERenderSettingControl.SLIDER,
    description: "How strongly raindrops ripple wet surfaces and puddles.",
    format: formatRainShare,
    isInert: isUnpuddled,
    label: "Ripples",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.rain.ripples, 0.05),
  },
};
