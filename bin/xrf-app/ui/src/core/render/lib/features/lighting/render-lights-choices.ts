import { formatNumber } from "@xrf/format";
import {
  ERenderLightFalloff,
  ERenderLightShadowFilter,
  ERenderLightShafts,
  RenderLightFalloff,
  RenderLightShadowFilter,
  RenderLightShafts,
} from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderLightsSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

const RENDER_LIGHT_FALLOFF_DESCRIPTIONS: Readonly<Record<RenderLightFalloff, string>> = {
  [ERenderLightFalloff.PHYSICAL]:
    "At the anchored light level lamps fall off as light does in the world: a pool about the bulb, dimmer towards " +
    "their reach, as much light within it as the game's.",
  [ERenderLightFalloff.ENGINE]:
    "At the anchored light level lamps fall off by the game's own curve: flat lamp-lit rooms.",
};

const RENDER_LIGHT_SHAFTS_DESCRIPTIONS: Readonly<Record<RenderLightShafts, string>> = {
  [ERenderLightShafts.ENGINE]:
    "The lamps the game gives shafts: those the level asks for, or every lamp under Anomaly's engine.",
  [ERenderLightShafts.SHADOWED]:
    "Every shadowed lamp, as Anomaly forces them on: shafts on a level whose lamps ask for none.",
};

const RENDER_LIGHT_SHADOW_FILTER_NAMES: Readonly<Record<RenderLightShadowFilter, string>> = {
  [ERenderLightShadowFilter.ENGINE]: "Engine",
  [ERenderLightShadowFilter.SOFT]: "Soft",
};

/**
 * @param falloff - A lamp falloff.
 * @returns What it does, in a sentence.
 */
export function explainRenderLightFalloff(falloff: RenderLightFalloff): string {
  return RENDER_LIGHT_FALLOFF_DESCRIPTIONS[falloff];
}

/**
 * @param shafts - Which lamps draw shafts.
 * @returns What it does, in a sentence.
 */
export function explainRenderLightShafts(shafts: RenderLightShafts): string {
  return RENDER_LIGHT_SHAFTS_DESCRIPTIONS[shafts];
}

/**
 * @param filter - How a lamp's shadow is filtered.
 * @returns Its name as the settings say it.
 */
export function describeRenderLightShadowFilter(filter: RenderLightShadowFilter): string {
  return RENDER_LIGHT_SHADOW_FILTER_NAMES[filter];
}

/** @returns A shaft strength as a share of the game's. */
export function formatShaftStrength(strength: number): string {
  return `${formatNumber(strength * 100, 0)}%`;
}

/** The lamp falloffs, in the order they are offered. */
export const RENDER_LIGHT_FALLOFF_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightFalloff>> = [
  { label: "Engine", value: ERenderLightFalloff.ENGINE },
  { label: "Physical", value: ERenderLightFalloff.PHYSICAL },
];

/** Which lamps draw shafts, in the order offered. */
export const RENDER_LIGHT_SHAFTS_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightShafts>> = [
  { label: "Engine", value: ERenderLightShafts.ENGINE },
  { label: "Every lamp", value: ERenderLightShafts.SHADOWED },
];

/** The filters a lamp's shadow is offered in, in the order they are offered. */
export const RENDER_LIGHT_SHADOW_FILTER_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderLightShadowFilter>> =
  Object.values(ERenderLightShadowFilter).map((value: RenderLightShadowFilter) => ({
    label: describeRenderLightShadowFilter(value),
    value,
  }));

/**
 * How each lamp setting is set, in the settings and the Lamps popover alike, whose shadows and shafts each head a
 * section of their values.
 */
export const RENDER_LIGHTS_SETTINGS: TRenderSettings<TRenderLightsSettings> = {
  falloff: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderLightFalloff,
    isEntryDescribed: true,
    label: "Falloff",
    options: RENDER_LIGHT_FALLOFF_OPTIONS,
  },
  isEnabled: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Off, only the sun and baked lighting remain.",
    label: "Lamps",
  },
  isLevelLights: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "The level file's static lights, already in its lightmaps, so they add on top. Game setting: r2_allow_r1_lights.",
    label: "Level lights",
  },
  isShadowed: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Shadows from lights the game marks as casting. Cached in an atlas and redrawn only on change.",
    label: "Shadows",
  },
  isVolumetric: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Shafts of light from the shadowed lamps. Game setting: r2_volumetric_lights.",
    entryLabel: "Shafts",
    label: "Lamp shafts",
  },
  shadowFilter: {
    control: ERenderSettingControl.CHOICE,
    description: "Engine: the game's fixed four-tap filter. Soft: Anomaly's PCSS, wider with distance from the caster.",
    entryLabel: "Filter",
    isInert: (lights: TRenderLightsSettings) => !lights.isShadowed,
    label: "Shadow filter",
    options: RENDER_LIGHT_SHADOW_FILTER_OPTIONS,
  },
  shafts: {
    control: ERenderSettingControl.CHOICE,
    entryLabel: "From",
    explain: explainRenderLightShafts,
    isEntryDescribed: true,
    isInert: (lights: TRenderLightsSettings) => !lights.isVolumetric,
    label: "Shafts from",
    options: RENDER_LIGHT_SHAFTS_OPTIONS,
  },
  shaftStrength: {
    control: ERenderSettingControl.SLIDER,
    description: "How strong every lamp's shafts are against the game's, in either volumetric light.",
    entryLabel: "Strength",
    format: formatShaftStrength,
    isInert: (lights: TRenderLightsSettings) => !lights.isVolumetric,
    label: "Shaft strength",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.lights.shaftStrength, 0.05),
  },
};
