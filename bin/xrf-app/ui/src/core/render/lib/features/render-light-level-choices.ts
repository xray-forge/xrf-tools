import { formatNumber } from "@xrf/format";
import { ERenderLightLevel, RenderLightLevel } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

const RENDER_LIGHT_LEVEL_DESCRIPTIONS: Readonly<Record<RenderLightLevel, string>> = {
  [ERenderLightLevel.AUTHORED]:
    "The weathers' and the lamps' own light, balanced against each other as the game authors it.",
  [ERenderLightLevel.PHYSICAL]:
    "Real-world light: the sun and moon through the atmosphere, the sky as bright as the world's, real lamps.",
};

/**
 * @param level - A light level.
 * @returns What it does, in a sentence.
 */
export function explainRenderLightLevel(level: RenderLightLevel): string {
  return RENDER_LIGHT_LEVEL_DESCRIPTIONS[level];
}

/**
 * The light levels, in the order they are offered: the physical ones shown but not offered outside dev mode.
 *
 * @param isPhysicalAvailable - Whether the physical light levels may be chosen (`isPhysicalLightLevelAvailable`).
 * @returns The options.
 */
export function getRenderLightLevelOptions(
  isPhysicalAvailable: boolean
): ReadonlyArray<IChoiceFormRowOption<RenderLightLevel>> {
  return [
    { label: "Authored", value: ERenderLightLevel.AUTHORED },
    { label: "Physical", value: ERenderLightLevel.PHYSICAL, isDisabled: !isPhysicalAvailable },
  ];
}

/**
 * @param level - A light level as chosen and kept.
 * @param isPhysicalAvailable - Whether the physical light levels may be drawn.
 * @returns The level drawn: the authored one where the physical levels may not be.
 */
export function toDrawnRenderLightLevel(level: RenderLightLevel, isPhysicalAvailable: boolean): RenderLightLevel {
  return level === ERenderLightLevel.PHYSICAL && !isPhysicalAvailable ? ERenderLightLevel.AUTHORED : level;
}

/** The sun tint slider's bounds and step. */
export const RENDER_SUN_TINT_LIMITS: IRenderLimits = toRenderLimits(RENDER_FEATURE_SCHEMA.sunTint, 0.05);

/** @returns A sun tint in hundredths. */
export function formatSunTint(value: number): string {
  return formatNumber(value, 2);
}
