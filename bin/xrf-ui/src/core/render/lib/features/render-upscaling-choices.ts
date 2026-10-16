import { formatNumber } from "@xrf/format";
import { Nullable } from "@xrf/types";

import {
  ERenderDlssAvailability,
  ERenderDlssPreset,
  ERenderScale,
  RenderDlssAvailability,
  RenderDlssPreset,
  RenderScale,
} from "@/core/ipc/types/xrf-renderer";
import { IRenderChoiceOption } from "@/core/render/lib/features/render-choice-option";
import { IRenderLimits, toRenderLimits } from "@/core/render/lib/features/render-limits";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/render-feature-schema";

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

/** The render scales, in the order they are offered. */
export const RENDER_SCALE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderScale>> = Object.values(ERenderScale).map(
  (value: RenderScale) => ({ label: describeRenderScale(value), value })
);

const DLSS_UNAVAILABILITY: Readonly<Record<RenderDlssAvailability, Nullable<string>>> = {
  [ERenderDlssAvailability.AVAILABLE]: null,
  [ERenderDlssAvailability.UNBUILT]: "DLSS is not in this build: build with the dlss feature.",
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
  [ERenderDlssAvailability.UNBUILT]: "DLSS is not in this build: build with the dlss feature.",
  [ERenderDlssAvailability.UNSUPPORTED]: "Needs an NVIDIA RTX GPU and a driver with DLSS ray reconstruction.",
};

/**
 * @param availability - Whether the GPU drawing a view reconstructs rays with DLSS.
 * @param isLinear - Whether the scene holds linear light, as the Enhanced tonemap keeps it.
 * @returns Why ray reconstruction cannot be turned on there, said so the reader knows what would let it; none where
 *   it can.
 */
export function describeRayReconstructionUnavailability(
  availability: RenderDlssAvailability,
  isLinear: boolean
): Nullable<string> {
  return (
    RAY_RECONSTRUCTION_UNAVAILABILITY[availability] ??
    (isLinear ? null : "Needs the Enhanced tonemap: ray reconstruction works on linear light.")
  );
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
export const RENDER_DLSS_PRESET_SHORT_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderDlssPreset>> = Object.values(
  ERenderDlssPreset
).map((value: RenderDlssPreset) => ({
  label: value === ERenderDlssPreset.DEFAULT ? "Default" : value.toUpperCase(),
  value,
}));

/** DLSS's models, in the order they are offered. */
export const RENDER_DLSS_PRESET_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderDlssPreset>> = Object.values(
  ERenderDlssPreset
).map((value: RenderDlssPreset) => ({ label: RENDER_DLSS_PRESET_NAMES[value], value }));
