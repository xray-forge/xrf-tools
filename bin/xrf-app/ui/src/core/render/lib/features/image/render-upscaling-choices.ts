import { formatNumber } from "@xrf/format";
import {
  ERenderAntialiasing,
  ERenderDlssAvailability,
  ERenderDlssPreset,
  ERenderLightModel,
  ERenderRayReconstructionPreset,
  ERenderScale,
  ERenderTonemapMode,
  RenderDlssAvailability,
  RenderDlssPreset,
  RenderRayReconstructionPreset,
  RenderScale,
} from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import {
  IRenderFeatureSettings,
  TRenderUpscalingSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

/** How far the upscaled frame's sharpening goes. */
export const RENDER_SHARPENING_LIMITS: IRenderLimits = toRenderLimits(RENDER_FEATURE_SCHEMA.upscaling.sharpening, 0.05);

/** @returns How much the upscaled frame is sharpened. */
export function formatSharpening(sharpening: number): string {
  return formatNumber(sharpening, 2);
}

const RENDER_SCALE_NAMES: Readonly<Record<RenderScale, string>> = {
  [ERenderScale.NATIVE]: "Native",
  [ERenderScale.ULTRA_QUALITY]: "Ultra quality",
  [ERenderScale.QUALITY]: "Quality",
  [ERenderScale.BALANCED]: "Balanced",
  [ERenderScale.PERFORMANCE]: "Performance",
  [ERenderScale.ULTRA_PERFORMANCE]: "Ultra performance",
};

/**
 * @param scale - A render scale.
 * @returns Its name as the settings say it, with the share of each side it draws.
 */
export function describeRenderScale(scale: RenderScale): string {
  return RENDER_SCALE_NAMES[scale];
}

/** The render scales, the smallest first as every ordered choice is offered, native last. */
export const RENDER_SCALE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderScale>> = [...Object.values(ERenderScale)]
  .reverse()
  .map((value: RenderScale) => ({ label: describeRenderScale(value), value }));

const DLSS_UNAVAILABILITY: Readonly<Record<RenderDlssAvailability, Nullable<string>>> = {
  [ERenderDlssAvailability.AVAILABLE]: null,
  [ERenderDlssAvailability.UNBUILT]: "This build has no DLSS.",
  [ERenderDlssAvailability.UNSUPPORTED]: "DLSS needs an NVIDIA RTX GPU and a current driver.",
};

/**
 * @param availability - Whether the GPU drawing a view upscales with DLSS.
 * @returns Why DLSS cannot be picked there, said so the reader knows what would let it; none where it can.
 */
export function describeDlssUnavailability(availability: RenderDlssAvailability): Nullable<string> {
  return DLSS_UNAVAILABILITY[availability];
}

const RAY_RECONSTRUCTION_UNAVAILABILITY: Readonly<Record<RenderDlssAvailability, Nullable<string>>> = {
  [ERenderDlssAvailability.AVAILABLE]: null,
  [ERenderDlssAvailability.UNBUILT]: "This build has no DLSS.",
  [ERenderDlssAvailability.UNSUPPORTED]: "Needs an NVIDIA RTX GPU and a driver with DLSS ray reconstruction.",
};

/**
 * @param availability - Whether the GPU drawing a view reconstructs rays with DLSS.
 * @param isLinear - Whether the scene holds linear light, as the Enhanced tonemap and the physical light keep it.
 * @returns Why ray reconstruction cannot be turned on there, said so the reader knows what would let it; none where
 *   it can.
 */
export function describeRayReconstructionUnavailability(
  availability: RenderDlssAvailability,
  isLinear: boolean
): Nullable<string> {
  return (
    RAY_RECONSTRUCTION_UNAVAILABILITY[availability] ??
    (isLinear ? null : "Needs the Enhanced tonemap or the physical light: ray reconstruction works on linear light.")
  );
}

/**
 * @param features - Every feature as drawn.
 * @returns Whether the scene holds linear light, which ray reconstruction needs: the Enhanced tonemap's or the physical
 *   light's, as the renderer decides it (`RenderViewOptions::is_scene_linear`).
 */
export function isRenderSceneLinear(features: IRenderFeatureSettings): boolean {
  return features.tonemap === ERenderTonemapMode.ENHANCED || features.lightModel === ERenderLightModel.PHYSICAL;
}

const RENDER_DLSS_PRESET_NAMES: Readonly<Record<RenderDlssPreset, string>> = {
  [ERenderDlssPreset.DEFAULT]: "Default",
  [ERenderDlssPreset.J]: "J: transformer",
  [ERenderDlssPreset.K]: "K: transformer, best",
  [ERenderDlssPreset.L]: "L: ultra performance",
  [ERenderDlssPreset.M]: "M: performance",
};

const RENDER_DLSS_PRESET_DETAILS: Readonly<Record<RenderDlssPreset, string>> = {
  [ERenderDlssPreset.DEFAULT]: "NVIDIA's pick by quality: K up to Balanced, M at Performance, L at Ultra performance.",
  [ERenderDlssPreset.J]: "Transformer, like K, with slightly less ghosting and more flicker.",
  [ERenderDlssPreset.K]: "Transformer: the sharpest image, at the highest cost.",
  [ERenderDlssPreset.L]: "NVIDIA's model for Ultra performance.",
  [ERenderDlssPreset.M]: "NVIDIA's model for Performance.",
};

/**
 * @param preset - A DLSS model.
 * @returns What it is, in a sentence.
 */
export function describeDlssPreset(preset: RenderDlssPreset): string {
  return RENDER_DLSS_PRESET_DETAILS[preset];
}

/** DLSS's models by their letters alone, short enough for a toolbar popover's row. */
export const RENDER_DLSS_PRESET_SHORT_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderDlssPreset>> = Object.values(
  ERenderDlssPreset
).map((value: RenderDlssPreset) => ({
  label: value === ERenderDlssPreset.DEFAULT ? "Default" : value.toUpperCase(),
  value,
}));

const RENDER_RAY_RECONSTRUCTION_PRESET_DETAILS: Readonly<Record<RenderRayReconstructionPreset, string>> = {
  [ERenderRayReconstructionPreset.DEFAULT]:
    "NVIDIA's pick, which a driver update may change; E under the physical light, where F blotches dark foliage and D " +
    "holds a moving sun's glint on water.",
  [ERenderRayReconstructionPreset.D]: "DLSS 4's transformer model, the one games ship.",
  [ERenderRayReconstructionPreset.E]: "A further model of DLSS 4's, which NVIDIA's guides do not describe.",
  [ERenderRayReconstructionPreset.F]: "DLSS 4.5's second-generation transformer model, NVIDIA's pick now.",
};

/**
 * @param preset - A ray reconstruction model.
 * @returns What it is, in a sentence.
 */
export function describeRayReconstructionPreset(preset: RenderRayReconstructionPreset): string {
  return RENDER_RAY_RECONSTRUCTION_PRESET_DETAILS[preset];
}

/** Ray reconstruction's models by their letters, in the order they are offered. */
export const RENDER_RAY_RECONSTRUCTION_PRESET_OPTIONS: ReadonlyArray<
  IChoiceFormRowOption<RenderRayReconstructionPreset>
> = Object.values(ERenderRayReconstructionPreset).map((value: RenderRayReconstructionPreset) => ({
  label: value === ERenderRayReconstructionPreset.DEFAULT ? "Default" : value.toUpperCase(),
  value,
}));

/** DLSS's models, in the order they are offered. */
export const RENDER_DLSS_PRESET_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderDlssPreset>> = Object.values(
  ERenderDlssPreset
).map((value: RenderDlssPreset) => ({ label: RENDER_DLSS_PRESET_NAMES[value], value }));

/** Whether DLSS resolves the frame, which none of its values tune otherwise. */
function isUnresolvedByDlss(features: IRenderFeatureSettings): boolean {
  return features.antialiasing !== ERenderAntialiasing.DLSS;
}

/**
 * How each upscaling setting is set, in the settings and the Image quality popover alike. Whether a frame is drawn
 * smaller than its viewport, which sharpening applies to, and what the GPU can do are the surfaces' to tell their rows.
 */
export const RENDER_UPSCALING_SETTINGS: TRenderSettings<TRenderUpscalingSettings> = {
  dlssPreset: {
    control: ERenderSettingControl.CHOICE,
    explain: describeDlssPreset,
    isInert: (upscaling: TRenderUpscalingSettings, features: IRenderFeatureSettings) =>
      isUnresolvedByDlss(features) || upscaling.isRayReconstructed,
    label: "DLSS model",
    options: RENDER_DLSS_PRESET_OPTIONS,
  },
  isRayReconstructed: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Denoises the traced light as DLSS upscales. Needs the Enhanced tonemap or the physical light, and an RTX GPU " +
      "with ray reconstruction.",
    isInert: (_: TRenderUpscalingSettings, features: IRenderFeatureSettings) =>
      isUnresolvedByDlss(features) || !isRenderSceneLinear(features),
    label: "DLSS ray reconstruction",
  },
  rayReconstructionPreset: {
    control: ERenderSettingControl.CHOICE,
    explain: describeRayReconstructionPreset,
    isInert: (upscaling: TRenderUpscalingSettings, features: IRenderFeatureSettings) =>
      isUnresolvedByDlss(features) || !upscaling.isRayReconstructed,
    label: "Ray reconstruction model",
    options: RENDER_RAY_RECONSTRUCTION_PRESET_OPTIONS,
  },
  scale: {
    control: ERenderSettingControl.CHOICE,
    description:
      "Share of the output resolution the scene is drawn at. TAA, FSR and DLSS upscale temporally; other modes use " +
      "FSR 1.",
    entryLabel: "Quality",
    isStacked: true,
    label: "Upscaling quality",
    options: RENDER_SCALE_OPTIONS,
  },
  sharpening: {
    control: ERenderSettingControl.SLIDER,
    description:
      "RCAS sharpening of a frame drawn smaller than the viewport or resolved temporally (TAA, FSR, DLSS). " +
      "Inactive otherwise.",
    format: formatSharpening,
    label: "Sharpening",
    limits: RENDER_SHARPENING_LIMITS,
  },
};
