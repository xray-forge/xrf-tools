import { formatNumber } from "@xrf/format";
import { ERenderLightLevel, RenderLightLevel } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

const RENDER_LIGHT_LEVEL_DESCRIPTIONS: Readonly<Record<RenderLightLevel, string>> = {
  [ERenderLightLevel.AUTHORED]:
    "The weathers' and the lamps' own light, balanced against each other as the game authors it.",
  [ERenderLightLevel.ANCHORED]:
    "The authored light made additive: each light anchored to the game's look where it shines alone, adding linearly where several do.",
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
 * The light levels, in the order they are offered: those past the authored shown but not offered outside dev mode.
 *
 * @param isDevAvailable - Whether the dev light levels may be chosen (`isDevLightLevelAvailable`).
 * @returns The options.
 */
export function getRenderLightLevelOptions(
  isDevAvailable: boolean
): ReadonlyArray<IChoiceFormRowOption<RenderLightLevel>> {
  return [
    { label: "Authored", value: ERenderLightLevel.AUTHORED },
    { label: "Anchored", value: ERenderLightLevel.ANCHORED, isDisabled: !isDevAvailable },
    { label: "Physical", value: ERenderLightLevel.PHYSICAL, isDisabled: !isDevAvailable },
  ];
}

/**
 * @param level - A light level as chosen and kept.
 * @param isDevAvailable - Whether the dev light levels may be drawn.
 * @returns The level drawn: the authored one where a dev level may not be.
 */
export function toDrawnRenderLightLevel(level: RenderLightLevel, isDevAvailable: boolean): RenderLightLevel {
  return level !== ERenderLightLevel.AUTHORED && !isDevAvailable ? ERenderLightLevel.AUTHORED : level;
}

/** The sun tint slider's bounds and step. */
export const RENDER_SUN_TINT_LIMITS: IRenderLimits = toRenderLimits(RENDER_FEATURE_SCHEMA.sunTint, 0.05);

/** @returns A sun tint in hundredths. */
export function formatSunTint(value: number): string {
  return formatNumber(value, 2);
}
