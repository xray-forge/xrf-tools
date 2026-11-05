import { formatNumber, formatPercent } from "@xrf/format";
import { ERenderContactShadowMode, RenderContactShadowMode } from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { TRenderContactShadowSettings } from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

/** @returns How far a contact shadow's ray reaches towards the sun or a light, in metres. */
export function formatContactShadowLength(length: number): string {
  return `${formatNumber(length, 2)} m`;
}

/** @returns How much light a contact shadow takes away. */
export function formatContactShadowIntensity(intensity: number): string {
  return formatPercent(intensity);
}

/** @returns How deep what the depth shows is taken to be, in centimetres. */
export function formatContactShadowThickness(thickness: number): string {
  return `${formatNumber(thickness * 100, 0)} cm`;
}

/** @returns How many depth reads a contact shadow's ray takes. */
export function formatContactShadowSteps(steps: number): string {
  return formatNumber(steps, 0);
}

/**
 * @param lights - Local lights each pixel marches contact shadows towards at most, the strongest there.
 * @returns It as the popover reads it: a count of lights, or the sun's alone for none.
 */
export function formatContactShadowLights(lights: number): string {
  return lights ? `${lights} light${lights > 1 ? "s" : ""}` : "Sun only";
}

const RENDER_CONTACT_SHADOW_MODE_DESCRIPTIONS: Readonly<Record<RenderContactShadowMode, string>> = {
  [ERenderContactShadowMode.ENGINE]: "Shadows by the sun's shadow maps alone, as the game does.",
  [ERenderContactShadowMode.ENHANCED]:
    "Finds the small shadows the maps miss by marching each pixel towards the sun over the frame's depth.",
};

/**
 * @param mode - A contact shadow mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderContactShadowMode(mode: RenderContactShadowMode): string {
  return RENDER_CONTACT_SHADOW_MODE_DESCRIPTIONS[mode];
}

/** The contact shadow modes, in the order they are offered. */
export const RENDER_CONTACT_SHADOW_MODE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderContactShadowMode>> = [
  { label: "Engine", value: ERenderContactShadowMode.ENGINE },
  { label: "Enhanced", value: ERenderContactShadowMode.ENHANCED },
];

/** Whether the maps alone shadow the sun, which none of the contact shadows' values tune. */
function isEngineContactShadow(contact: TRenderContactShadowSettings): boolean {
  return contact.mode !== ERenderContactShadowMode.ENHANCED;
}

/**
 * How each contact shadow setting is set, in the settings' Shadows tab and the Sun shadows popover alike, each under a
 * Contact shadows heading.
 */
export const RENDER_CONTACT_SHADOW_SETTINGS: TRenderSettings<TRenderContactShadowSettings> = {
  intensity: {
    control: ERenderSettingControl.SLIDER,
    description: "How much of the light a contact shadow takes away.",
    format: formatContactShadowIntensity,
    isInert: isEngineContactShadow,
    label: "Intensity",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.intensity, 0.05),
  },
  isFullResolution: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Marches every pixel rather than one in four: thin foliage stops flickering, at four times the cost.",
    isEntryDescribed: true,
    isInert: isEngineContactShadow,
    label: "Full resolution",
  },
  length: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres a pixel's ray reaches towards the sun or a lamp over the frame's depth.",
    format: formatContactShadowLength,
    isInert: isEngineContactShadow,
    label: "Length",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.length, 0.05),
  },
  lights: {
    control: ERenderSettingControl.SLIDER,
    description: "Lamps each pixel marches towards too, the strongest there; none marches towards the sun alone.",
    format: formatContactShadowLights,
    isInert: isEngineContactShadow,
    label: "Lamps",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.lights, 1),
  },
  mode: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderContactShadowMode,
    isEntryDescribed: true,
    label: "Mode",
    options: RENDER_CONTACT_SHADOW_MODE_OPTIONS,
  },
  steps: {
    control: ERenderSettingControl.SLIDER,
    description: "Depth reads along each ray: more finds smaller shadows and costs more.",
    format: formatContactShadowSteps,
    isInert: isEngineContactShadow,
    label: "Steps",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.steps, 1),
  },
  thickness: {
    control: ERenderSettingControl.SLIDER,
    description: "How deep what the frame's depth shows is taken to be: light passes behind anything thinner.",
    format: formatContactShadowThickness,
    isInert: isEngineContactShadow,
    label: "Thickness",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.shadows.contact.thickness, 0.01),
  },
};
