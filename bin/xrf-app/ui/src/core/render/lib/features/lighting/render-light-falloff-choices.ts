import { ERenderLightFalloff, RenderLightFalloff } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

const RENDER_LIGHT_FALLOFF_DESCRIPTIONS: Readonly<Record<RenderLightFalloff, string>> = {
  [ERenderLightFalloff.PHYSICAL]:
    "At the anchored light level lamps fall off as light does in the world: a pool about the bulb, dimmer towards " +
    "their reach, as much light within it as the game's.",
  [ERenderLightFalloff.ENGINE]:
    "At the anchored light level lamps fall off by the game's own curve: flat lamp-lit rooms.",
};

/**
 * @param falloff - A lamp falloff.
 * @returns What it does, in a sentence.
 */
export function explainRenderLightFalloff(falloff: RenderLightFalloff): string {
  return RENDER_LIGHT_FALLOFF_DESCRIPTIONS[falloff];
}

/** The lamp falloffs, in the order they are offered. */
export const RENDER_LIGHT_FALLOFF_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightFalloff>> = [
  { label: "Physical", value: ERenderLightFalloff.PHYSICAL },
  { label: "Engine", value: ERenderLightFalloff.ENGINE },
];
