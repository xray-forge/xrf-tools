import { formatNumber } from "@xrf/format";
import { ERenderLightLevel, RenderLightLevel } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

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

/** The light levels, in the order they are offered. */
export const RENDER_LIGHT_LEVEL_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightLevel>> = [
  { label: "Authored", value: ERenderLightLevel.AUTHORED },
  { label: "Anchored", value: ERenderLightLevel.ANCHORED },
  { label: "Physical", value: ERenderLightLevel.PHYSICAL },
];

/** @returns A sun tint in hundredths. */
export function formatSunTint(value: number): string {
  return formatNumber(value, 2);
}
