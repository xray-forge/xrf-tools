import { ERenderLightModel, RenderLightModel } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/ChoiceFormRow/choice-form-row-option";

const RENDER_LIGHT_MODEL_DESCRIPTIONS: Readonly<Record<RenderLightModel, string>> = {
  [ERenderLightModel.ENGINE]: "Lights textures and colours as the stored display values they are, as the game does.",
  [ERenderLightModel.PHYSICAL]:
    "Lights in linear light: textures, the weather's and the lamps' colours decoded first, the frame encoded for the " +
    "display once it is tonemapped. In development: its light ratios and tonemap are not yet calibrated.",
};

/**
 * @param model - A light model.
 * @returns What it does, in a sentence or two.
 */
export function explainRenderLightModel(model: RenderLightModel): string {
  return RENDER_LIGHT_MODEL_DESCRIPTIONS[model];
}

/** The light models, in the order they are offered. */
export const RENDER_LIGHT_MODEL_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightModel>> = [
  { label: "Engine", value: ERenderLightModel.ENGINE },
  { label: "Physical", value: ERenderLightModel.PHYSICAL },
];
