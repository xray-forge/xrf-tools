import { formatCount, formatNumber, formatPercent } from "@xrf/format";
import {
  ERenderAmbientOcclusionMethod,
  RenderAmbientOcclusionMethod,
  RenderAmbientOcclusionSettings,
} from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import {
  TRenderAmbientOcclusionGtaoSettings,
  TRenderAmbientOcclusionRtaoSettings,
  TRenderAmbientOcclusionSettings,
  TRenderAmbientOcclusionVbaoSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

const RENDER_AMBIENT_OCCLUSION_METHOD_NAMES: Readonly<Record<RenderAmbientOcclusionMethod, string>> = {
  [ERenderAmbientOcclusionMethod.GTAO]: "GTAO",
  [ERenderAmbientOcclusionMethod.VBAO]: "VBAO",
  [ERenderAmbientOcclusionMethod.RTAO]: "RTAO",
};

const RENDER_AMBIENT_OCCLUSION_METHOD_DESCRIPTIONS: Readonly<Record<RenderAmbientOcclusionMethod, string>> = {
  [ERenderAmbientOcclusionMethod.GTAO]:
    "Darkens all that lies behind whatever stands in front, however thin: XeGTAO's horizons.",
  [ERenderAmbientOcclusionMethod.VBAO]:
    "Takes every occluder as a thickness deep, so light passes behind grass, fences and branches.",
  [ERenderAmbientOcclusionMethod.RTAO]:
    "Traces rays against the level's geometry, so what is off screen or hidden occludes too; grass occludes nothing.",
};

/**
 * @param method - An ambient occlusion method.
 * @returns Its name, as the popover offers it.
 */
export function describeRenderAmbientOcclusionMethod(method: RenderAmbientOcclusionMethod): string {
  return RENDER_AMBIENT_OCCLUSION_METHOD_NAMES[method];
}

/**
 * @param method - An ambient occlusion method.
 * @returns What it does, in a sentence.
 */
export function explainRenderAmbientOcclusionMethod(method: RenderAmbientOcclusionMethod): string {
  return RENDER_AMBIENT_OCCLUSION_METHOD_DESCRIPTIONS[method];
}

/** The ambient occlusion methods, in the order they are offered. */
export const RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS: ReadonlyArray<
  IChoiceFormRowOption<RenderAmbientOcclusionMethod>
> = Object.values(ERenderAmbientOcclusionMethod).map((value: RenderAmbientOcclusionMethod) => ({
  label: describeRenderAmbientOcclusionMethod(value),
  value,
}));

/** The same where the GPU traces no rays: RTAO greyed out, VBAO searching in its place. */
export const RENDER_AMBIENT_OCCLUSION_METHOD_UNTRACED_OPTIONS: ReadonlyArray<
  IChoiceFormRowOption<RenderAmbientOcclusionMethod>
> = RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS.map((option: IChoiceFormRowOption<RenderAmbientOcclusionMethod>) =>
  option.value === ERenderAmbientOcclusionMethod.RTAO ? { ...option, isDisabled: true } : option
);

/** @returns How deep an occluder is taken to be, in metres. */
export function formatOcclusionThickness(thickness: number): string {
  return `${formatNumber(thickness, 2)} m`;
}

/** @returns How much of the light bounced between a crease's sides comes back. */
export function formatOcclusionBounce(bounce: number): string {
  return formatPercent(bounce);
}

/** @returns Over how many frames the occlusion is averaged, or that it is not. */
export function formatOcclusionAccumulation(accumulation: number): string {
  return accumulation > 1 ? `${accumulation} frames` : "None";
}

/** @returns An occlusion radius, in metres. */
export function formatOcclusionRadius(radius: number): string {
  return `${formatNumber(radius, 2)} m`;
}

/** @returns An occlusion strength. */
export function formatOcclusionStrength(strength: number): string {
  return formatNumber(strength, 1);
}

/**
 * @param occlusion - The ambient occlusion's settings.
 * @returns How its method searches, in a few words: GTAO's and VBAO's slices by steps, RTAO's rays.
 */
export function describeRenderAmbientOcclusionSearch(
  occlusion: Pick<RenderAmbientOcclusionSettings, "gtao" | "method" | "rtao" | "vbao">
): string {
  if (occlusion.method === ERenderAmbientOcclusionMethod.RTAO) {
    return `${formatCount(occlusion.rtao.rays)} rays`;
  }

  const search: { slices: number; steps: number } =
    occlusion.method === ERenderAmbientOcclusionMethod.GTAO ? occlusion.gtao : occlusion.vbao;

  return `${formatCount(search.slices)} × ${formatCount(search.steps)} steps`;
}

/**
 * How each ambient occlusion setting is set, in the settings and the Ambient light popover alike. Which method's
 * values apply turns on the method and on whether the GPU traces rays, which the popover tells its entries itself.
 */
export const RENDER_AMBIENT_OCCLUSION_SETTINGS: TRenderSettings<TRenderAmbientOcclusionSettings> = {
  gtao: { control: ERenderSettingControl.CUSTOM, label: "GTAO" },
  isEnabled: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Off, only baked occlusion remains.",
    label: "Ambient occlusion",
  },
  method: {
    control: ERenderSettingControl.CHOICE,
    explain: explainRenderAmbientOcclusionMethod,
    label: "Method",
    options: RENDER_AMBIENT_OCCLUSION_METHOD_OPTIONS,
  },
  radius: {
    control: ERenderSettingControl.SLIDER,
    description: "Occlusion radius in metres.",
    format: formatOcclusionRadius,
    label: "Radius",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.radius, 0.1),
  },
  rtao: { control: ERenderSettingControl.CUSTOM, label: "RTAO" },
  strength: {
    control: ERenderSettingControl.SLIDER,
    description: "Occlusion intensity. 1 is each method's own curve.",
    format: formatOcclusionStrength,
    label: "Strength",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.strength, 0.1),
  },
  vbao: { control: ERenderSettingControl.CUSTOM, label: "VBAO" },
};

/** How each of GTAO's search values is set; the popover's one pair of search values names no method. */
export const RENDER_AMBIENT_OCCLUSION_GTAO_SETTINGS: TRenderSettings<TRenderAmbientOcclusionGtaoSettings> = {
  slices: {
    control: ERenderSettingControl.SLIDER,
    description: "Directions GTAO searches about a pixel for its horizons.",
    entryLabel: "Slices",
    format: formatCount,
    label: "GTAO slices",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.gtao.slices, 1),
  },
  steps: {
    control: ERenderSettingControl.SLIDER,
    description: "Steps each way along each of GTAO's slices.",
    entryLabel: "Steps",
    format: formatCount,
    label: "GTAO steps",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.gtao.steps, 1),
  },
};

/** How each of VBAO's values is set: its search, and what it and RTAO gather over frames. */
export const RENDER_AMBIENT_OCCLUSION_VBAO_SETTINGS: TRenderSettings<TRenderAmbientOcclusionVbaoSettings> = {
  accumulation: {
    control: ERenderSettingControl.SLIDER,
    format: formatOcclusionAccumulation,
    label: "Accumulation",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.accumulation, 1),
  },
  bounce: {
    control: ERenderSettingControl.SLIDER,
    format: formatOcclusionBounce,
    label: "Bounce",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.bounce, 0.05),
  },
  slices: {
    control: ERenderSettingControl.SLIDER,
    description: "Directions VBAO searches about a pixel, its visibility bitmask's.",
    entryLabel: "Slices",
    format: formatCount,
    label: "VBAO slices",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.slices, 1),
  },
  steps: {
    control: ERenderSettingControl.SLIDER,
    description: "Steps along each of VBAO's slices.",
    entryLabel: "Steps",
    format: formatCount,
    label: "VBAO steps",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.steps, 1),
  },
  thickness: {
    control: ERenderSettingControl.SLIDER,
    format: formatOcclusionThickness,
    label: "Thickness",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.thickness, 0.01),
  },
};

/** How RTAO's rays are set. */
export const RENDER_AMBIENT_OCCLUSION_RTAO_SETTINGS: TRenderSettings<TRenderAmbientOcclusionRtaoSettings> = {
  rays: {
    control: ERenderSettingControl.SLIDER,
    description: "Rays RTAO traces from each point a frame, on a GPU that traces rays.",
    entryLabel: "Rays",
    format: formatCount,
    label: "RTAO rays",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.rtao.rays, 1),
  },
};
