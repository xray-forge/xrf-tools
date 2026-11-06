import { formatCount, formatNumber, formatPercent } from "@xrf/format";
import {
  ERenderAntialiasing,
  ERenderReflectionMode,
  ERenderReflectionProbe,
  ERenderWaterMode,
  RenderReflectionMode,
  RenderReflectionProbe,
} from "@xrf/ipc/types/xrf-renderer";
import { IChoiceFormRowOption } from "@xrf/ui/form/choice-row";

import { toRenderLimits } from "@/core/render/lib/features/render-limits";
import { ERenderSettingControl, TRenderSettings } from "@/core/render/lib/features/render-setting";
import { RENDER_TRACE_SIZE_OPTIONS } from "@/core/render/lib/features/tracing/render-trace-size-choices";
import {
  IRenderFeatureSettings,
  TRenderReflectionSettings,
} from "@/core/render/lib/settings/choice/render-feature-settings";
import { RENDER_FEATURE_SCHEMA } from "@/core/render/lib/settings/schema/render-feature-schema";

const RENDER_REFLECTION_MODE_DESCRIPTIONS: Readonly<Record<RenderReflectionMode, string>> = {
  [ERenderReflectionMode.ENGINE]:
    "The ground, puddles and glossy surfaces reflect the sky's cube alone, as the game does",
  [ERenderReflectionMode.ENHANCED]:
    "The ground, puddles and glossy surfaces reflect what their rays meet on the screen, the sky's cube elsewhere",
};

/**
 * @param mode - A reflection mode.
 * @returns What it does, in a sentence.
 */
export function explainRenderReflectionMode(mode: RenderReflectionMode): string {
  return RENDER_REFLECTION_MODE_DESCRIPTIONS[mode];
}

const RENDER_REFLECTION_PROBE_NAMES: Readonly<Record<RenderReflectionProbe, string>> = {
  [ERenderReflectionProbe.COARSE]: "256",
  [ERenderReflectionProbe.FINE]: "512",
  [ERenderReflectionProbe.FINEST]: "1024",
};

/** The surroundings cube sizes, coarsest first. */
export const RENDER_REFLECTION_PROBE_OPTIONS: ReadonlyArray<IChoiceFormRowOption<RenderReflectionProbe>> =
  Object.values(ERenderReflectionProbe).map((value: RenderReflectionProbe) => ({
    label: RENDER_REFLECTION_PROBE_NAMES[value],
    value,
  }));

/** @returns What a surface's share of reflection is scaled by. */
export function formatReflectionIntensity(intensity: number): string {
  return formatPercent(intensity);
}

/** @returns How far a ray is traced. */
export function formatReflectionDistance(distance: number): string {
  return `${formatNumber(distance, 0)} m`;
}

/** @returns How far the sky's highlights are expanded as their light. */
export function formatReflectionSkyGain(gain: number): string {
  return `${formatNumber(gain, 2)}×`;
}

/** @returns How many frames a pixel the screen misses waits before it is traced again, its row naming the unit. */
export function formatReflectionTraceInterval(interval: number): string {
  return formatCount(Math.max(1, interval));
}

/** @returns The roughness past which a surface is not traced. */
export function formatReflectionTraceRoughness(roughness: number): string {
  return formatNumber(roughness, 2);
}

/** Whether the surfaces reflect the sky's cube alone, which none of their tracing's values tune. */
function isEngineReflection(reflections: TRenderReflectionSettings): boolean {
  return reflections.mode !== ERenderReflectionMode.ENHANCED;
}

/** Whether ray reconstruction is asked for, which takes every pixel's own ray each frame. */
function isRayReconstructionAsked(features: IRenderFeatureSettings): boolean {
  return features.antialiasing === ERenderAntialiasing.DLSS && features.upscaling.isRayReconstructed;
}

/**
 * How each reflection setting is set, in the settings (Reflections, Water reflection and Ray-traced reflections) and
 * the Reflections popover alike. The popover reads ray reconstruction and tracing off the GPU and the frame drawn,
 * which it tells its entries itself.
 */
export const RENDER_REFLECTION_SETTINGS: TRenderSettings<TRenderReflectionSettings> = {
  distance: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres a ray goes at most, the water's traced rays too.",
    format: formatReflectionDistance,
    isInert: isEngineReflection,
    label: "Distance",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.distance, 5),
  },
  intensity: {
    control: ERenderSettingControl.SLIDER,
    description: "How much surfaces reflect: 1 as their material does.",
    format: formatReflectionIntensity,
    isInert: isEngineReflection,
    label: "Intensity",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.intensity, 0.05),
  },
  isDryTraced: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Traces a dry level's surfaces that show little of what they reflect, a dielectric's 4%; off, only " +
      "metal and glass-like surfaces trace until the rain wets the rest, which reflect the sky's cube.",
    isEntryDescribed: true,
    isInert: isEngineReflection,
    label: "Trace dry surfaces",
  },
  isGrassReflected: {
    control: ERenderSettingControl.CHECKBOX,
    description: "Grass stands in the reflections and stops the rays behind it.",
    isInert: isEngineReflection,
    label: "Reflect grass",
  },
  isRayBudgeted: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Traces fewer rays where a reflection is dim or rough, sharing them in each quad of pixels; saves most " +
      "at full size and under ray reconstruction, where every pixel would trace.",
    isEntryDescribed: true,
    isInert: isEngineReflection,
    label: "Ray budgets",
  },
  isTraced: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Traces what the screen misses against the level's geometry on a GPU that traces rays, for the " +
      "surfaces and the water alike; elsewhere they reflect the sky's cube.",
    label: "Ray-traced reflections",
  },
  isWaterFullyTraced: {
    control: ERenderSettingControl.CHECKBOX,
    description:
      "Traces every water pixel every frame, its rays meeting the grass near the eye, so cables, the undersides " +
      "of pipes and reeds reflect as they stand and are lit; the screen answers only plants further out. Costs " +
      "about a ray a water pixel a frame, and the grass's shadow cards where grass shadows are off.",
    isInert: (reflections: TRenderReflectionSettings, features: IRenderFeatureSettings) =>
      !reflections.isTraced || features.water.mode !== ERenderWaterMode.ENHANCED || !reflections.isWaterReflected,
    label: "Trace all water",
  },
  isWaterReflected: {
    control: ERenderSettingControl.CHECKBOX,
    description: (_: TRenderReflectionSettings, features: IRenderFeatureSettings) =>
      features.water.mode === ERenderWaterMode.ENHANCED
        ? "Off, the water reflects the sky's cube alone and keeps the rest of its look."
        : "The game's water reflects the sky's cube; set the water to Enhanced.",
    isInert: (_: TRenderReflectionSettings, features: IRenderFeatureSettings) =>
      features.water.mode !== ERenderWaterMode.ENHANCED,
    label: "Water reflects the screen",
  },
  // Set as the surfaces' toggle, which says what the mode drawn does.
  mode: {
    control: ERenderSettingControl.CUSTOM,
    label: "Screen-space reflections",
  },
  probe: {
    control: ERenderSettingControl.CHOICE,
    description:
      "How fine the cube is, in texels a side, that a ray meeting nothing on the screen reads; finer takes more memory.",
    isInert: isEngineReflection,
    label: "Surroundings",
    options: RENDER_REFLECTION_PROBE_OPTIONS,
  },
  reach: {
    control: ERenderSettingControl.SLIDER,
    description: "Metres out to which the rain wets the ground for the rays to trace; the game wets 25 m.",
    format: formatReflectionDistance,
    isInert: isEngineReflection,
    label: "Wet reach",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.reach, 25),
  },
  skyGain: {
    control: ERenderSettingControl.SLIDER,
    description: "How bright the sky's highlights are in the reflections: 1 as the sky shows them.",
    format: formatReflectionSkyGain,
    isInert: isEngineReflection,
    label: "Sky light",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.skyGain, 0.05),
  },
  steps: {
    control: ERenderSettingControl.SLIDER,
    description: "How many cells of the depth a ray walks at most; a rough surface's ray walks half.",
    format: formatCount,
    isInert: isEngineReflection,
    label: "Steps",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.steps, 8),
  },
  traceInterval: {
    control: ERenderSettingControl.SLIDER,
    description: (_: TRenderReflectionSettings, features: IRenderFeatureSettings) =>
      isRayReconstructionAsked(features)
        ? "How often a missed pixel traces again, the last answer held between; the surfaces trace every " +
          "frame under ray reconstruction."
        : "How often a missed pixel traces again, the last answer held between.",
    format: formatReflectionTraceInterval,
    isInert: (reflections: TRenderReflectionSettings) => !reflections.isTraced,
    label: "Frames per trace",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.traceInterval, 1),
  },
  traceRoughness: {
    control: ERenderSettingControl.SLIDER,
    description: "How rough a surface may be and still be traced; rougher ones reflect the sky. 0.4 traces less.",
    format: formatReflectionTraceRoughness,
    isInert: isEngineReflection,
    label: "Trace roughness",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.traceRoughness, 0.05),
  },
  traceSize: {
    control: ERenderSettingControl.CHOICE,
    description: (_: TRenderReflectionSettings, features: IRenderFeatureSettings) =>
      isRayReconstructionAsked(features)
        ? "Full size every frame under ray reconstruction, which takes each pixel's own ray."
        : "Half traces a ray for every four pixels; full, one for each.",
    isInert: (reflections: TRenderReflectionSettings, features: IRenderFeatureSettings) =>
      isEngineReflection(reflections) || isRayReconstructionAsked(features),
    label: "Trace size",
    options: RENDER_TRACE_SIZE_OPTIONS,
  },
  waterSteps: {
    control: ERenderSettingControl.SLIDER,
    description: "How many steps the water's reflection marches over the screen.",
    format: formatCount,
    isInert: (reflections: TRenderReflectionSettings, features: IRenderFeatureSettings) =>
      features.water.mode !== ERenderWaterMode.ENHANCED || !reflections.isWaterReflected,
    label: "March steps",
    limits: toRenderLimits(RENDER_FEATURE_SCHEMA.reflections.waterSteps, 4),
  },
};
