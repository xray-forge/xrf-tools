import { ERenderLightLevel, RenderLightLevel } from "@/core/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

const RENDER_LIGHT_LEVEL_DESCRIPTIONS: Readonly<Record<RenderLightLevel, string>> = {
  [ERenderLightLevel.AUTHORED]:
    "The weathers' and the lamps' own brightness: a night as bright as the game authors it.",
  [ERenderLightLevel.PHYSICAL]:
    "Real-world brightness: a moonlit night of a fraction of a lux, lamps as bright as real ones and unseen by day.",
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
