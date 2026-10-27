import { formatCount, formatNumber, formatPercent } from "@xrf/format";
import {
  ERenderAmbientOcclusionMethod,
  RenderAmbientOcclusionMethod,
  RenderAmbientOcclusionSettings,
} from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/ChoiceFormRow/choice-form-row-option";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

/** The bounds each ambient occlusion value is offered between. */
export const RENDER_AMBIENT_OCCLUSION_LIMITS = {
  gtaoSlices: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.gtao.slices, 1),
  gtaoSteps: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.gtao.steps, 1),
  radius: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.radius, 0.1),
  rays: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.rtao.rays, 1),
  strength: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.strength, 0.1),
} as const;

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

/** The bounds each VBAO value is offered between. */
export const RENDER_AMBIENT_OCCLUSION_VBAO_LIMITS = {
  accumulation: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.accumulation, 1),
  bounce: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.bounce, 0.05),
  slices: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.slices, 1),
  steps: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.steps, 1),
  thickness: toRenderLimits(RENDER_FEATURE_SCHEMA.ambientOcclusion.vbao.thickness, 0.01),
} as const;

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
