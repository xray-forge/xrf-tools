import { formatNumber } from "@xrf/format";

import { ERenderLightLevel, RenderLightLevel } from "@/core/ipc/types/xrf-renderer";
import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

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

/** The light levels, in the order they are offered. */
export const RENDER_LIGHT_LEVEL_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightLevel>> = [
  { label: "Authored", value: ERenderLightLevel.AUTHORED },
  { label: "Physical", value: ERenderLightLevel.PHYSICAL },
];

/** The sun tint slider's bounds and step. */
export const RENDER_SUN_TINT_LIMITS: IRenderLimits = toRenderLimits(RENDER_FEATURE_SCHEMA.sunTint, 0.05);

/** @returns A sun tint in hundredths. */
export function formatSunTint(value: number): string {
  return formatNumber(value, 2);
}
