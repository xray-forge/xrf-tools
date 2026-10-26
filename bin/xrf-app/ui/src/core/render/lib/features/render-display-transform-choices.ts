import { ERenderDisplayTransform, RenderDisplayTransform } from "@xrf/ipc/types/xrf-renderer";

import { IChoiceFormRowOption } from "@/core/ui/form/ChoiceFormRow/choice-form-row-option";

const RENDER_DISPLAY_TRANSFORM_DESCRIPTIONS: Readonly<Record<RenderDisplayTransform, string>> = {
  [ERenderDisplayTransform.ENGINE]:
    "The engine's curve over the light encoded as the engine's values are: the game as the engine shows it. Each colour channel clips on its own, so bright colours turn yellow and white.",
  [ERenderDisplayTransform.AGX]:
    "AgX: highlights roll off towards white without their hues shifting, as Blender shows a render.",
  [ERenderDisplayTransform.PBR_NEUTRAL]:
    "Khronos PBR Neutral: colours as authored up to a shoulder near white, for reading materials.",
};

/**
 * @param transform - A display transform.
 * @returns What it does, in a sentence.
 */
export function explainRenderDisplayTransform(transform: RenderDisplayTransform): string {
  return RENDER_DISPLAY_TRANSFORM_DESCRIPTIONS[transform];
}

/** The display transforms, in the order they are offered. */
export const RENDER_DISPLAY_TRANSFORM_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderDisplayTransform>> = [
  { label: "Engine", value: ERenderDisplayTransform.ENGINE },
  { label: "AgX", value: ERenderDisplayTransform.AGX },
  { label: "PBR Neutral", value: ERenderDisplayTransform.PBR_NEUTRAL },
];
