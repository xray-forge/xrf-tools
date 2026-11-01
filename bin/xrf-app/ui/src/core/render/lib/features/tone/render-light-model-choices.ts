import { ERenderLightLevel, ERenderLightModel, RenderLightModel } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import {
  explainRenderDisplayTransform,
  RENDER_DISPLAY_TRANSFORM_OPTIONS,
} from "@/core/render/lib/features/tone/render-display-transform-choices";
import {
  explainRenderTonemapMode,
  RENDER_TONEMAP_MODE_OPTIONS,
} from "@/core/render/lib/features/tone/render-exposure-choices";
import {
  explainRenderLightLevel,
  formatSunTint,
  getRenderLightLevelOptions,
} from "@/core/render/lib/features/tone/render-light-level-choices";
import { TRenderLightSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

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

/** Whether the light is the engine's, which has no levels, display transform or grading of its own. */
function isEngineLight(light: TRenderLightSettings): boolean {
  return light.lightModel !== ERenderLightModel.PHYSICAL;
}

/**
 * How the light and its display are set, in the settings and the Look popover alike; the popover names them within
 * its sections. Every light level is offered here; a surface narrows them outside dev mode.
 */
export const RENDER_LIGHT_SETTINGS: TRenderSettings<TRenderLightSettings> = {
  displayTransform: {
    control: ERenderSettingControl.CHOICE,
    entryLabel: "Transform",
    explain: explainRenderDisplayTransform,
    isInert: isEngineLight,
    label: "Display transform",
    options: RENDER_DISPLAY_TRANSFORM_OPTIONS,
  },
  lightLevel: {
    control: ERenderSettingControl.CHOICE,
    entryLabel: "Levels",
    explain: explainRenderLightLevel,
    isInert: isEngineLight,
    label: "Light levels",
    options: getRenderLightLevelOptions(true),
  },
  lightModel: {
    control: ERenderSettingControl.CHOICE,
    entryLabel: "Model",
    explain: explainRenderLightModel,
    label: "Light",
    options: RENDER_LIGHT_MODEL_OPTIONS,
  },
  sunTint: {
    control: ERenderSettingControl.SLIDER,
    description: "How much of the weather's own sun colour the real-world light levels keep in the sun's.",
    format: formatSunTint,
    isInert: (light: TRenderLightSettings) => isEngineLight(light) || light.lightLevel !== ERenderLightLevel.PHYSICAL,
    label: "Sun tint",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.sunTint, 0.05),
  },
  tonemap: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderTonemapMode,
    isInert: (light: TRenderLightSettings) => !isEngineLight(light),
    label: "Tonemap",
    options: RENDER_TONEMAP_MODE_OPTIONS,
  },
};
