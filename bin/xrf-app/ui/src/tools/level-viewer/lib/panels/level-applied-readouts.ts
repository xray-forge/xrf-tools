import { formatCount, formatNumber } from "@xrf/format";
import {
  ERenderAntialiasing,
  RenderAppliedEnvironment,
  RenderAppliedReport,
  RenderBloomMode,
  RenderViewOptions,
} from "@xrf/ipc/types/xrf-renderer";
import { Nullable } from "@xrf/types";

import { describeRenderAntialiasing } from "@/core/render/lib/features/image/render-antialiasing-choices";
import {
  describeRenderScale,
  RENDER_UPSCALING_SETTINGS,
} from "@/core/render/lib/features/image/render-upscaling-choices";
import { describeRenderLightShadowFilter } from "@/core/render/lib/features/lighting/render-lights-choices";
import { formatShadowFilter } from "@/core/render/lib/features/lighting/render-shadow-choices";
import { RENDER_BLOOM_MODE_OPTIONS } from "@/core/render/lib/features/tone/render-bloom-choices";
import {
  describeRenderAmbientOcclusionMethod,
  describeRenderAmbientOcclusionSearch,
} from "@/core/render/lib/features/tracing/render-ambient-occlusion-choices";
import { formatIndirectLightIntensity } from "@/core/render/lib/features/tracing/render-indirect-light-choices";
import { formatReflectionIntensity } from "@/core/render/lib/features/tracing/render-reflection-choices";
import { describeRenderTraceSize } from "@/core/render/lib/features/tracing/render-trace-size-choices";
import { formatFogDensity, formatFogHeight, formatFogShare } from "@/core/render/lib/features/world/render-fog-choices";
import { formatGrassDensity, formatGrassRadius } from "@/core/render/lib/features/world/render-grass-choices";

/** One line of what the renderer draws a level with. */
export interface ILevelAppliedReadout {
  label: string;
  /** Whether it reads as a column of figures. */
  isMonospace?: boolean;
  /**
   * @returns What the renderer applied, a value it changed from what the view asked saying so; none where the line
   *   says nothing of this report.
   */
  read: (applied: RenderAppliedReport, requested: RenderViewOptions) => Nullable<string>;
}

/** A section of the Renderer panel's applied readouts. */
export interface ILevelAppliedSection {
  title: string;
  /** Whether the report has anything for it, every report where none is said. */
  isShown?: (applied: RenderAppliedReport) => boolean;
  readouts: ReadonlyArray<ILevelAppliedReadout>;
}

/**
 * @param value - What the renderer applied, as it reads.
 * @param asked - What the view asked, read alike.
 * @returns The value, and what was asked where the renderer drew something else.
 */
export function toLevelAppliedChange(value: string, asked: string): string {
  return value === asked ? value : `${value} · asked ${asked}`;
}

/** Three channels as the panel reads them. */
function formatTriple(values: ReadonlyArray<Nullable<number>>): string {
  return values.map((value: Nullable<number>) => formatNumber(value ?? 0, 2)).join("  ");
}

/** A bloom mode's name, as the settings offer it. */
function describeBloomMode(mode: RenderBloomMode): string {
  return RENDER_BLOOM_MODE_OPTIONS.find((option) => option.value === mode)?.label ?? mode;
}

/** Shadow cascades' widths, nearest first. */
function formatCascades(cascades: ReadonlyArray<Nullable<number>>): string {
  return `${cascades.map((width: Nullable<number>) => formatNumber(width ?? 0, 0)).join(" / ")} m`;
}

/** The environment's lines, which only a report with an environment has. */
function readEnvironment(
  applied: RenderAppliedReport,
  read: (environment: RenderAppliedEnvironment) => string
): Nullable<string> {
  return applied.environment ? read(applied.environment) : null;
}

/**
 * What the renderer actually draws a level with, section by section, each line said against what the view asked
 * (`RenderViewOptions`) wherever the renderer may draw something else.
 */
export const LEVEL_APPLIED_SECTIONS: ReadonlyArray<ILevelAppliedSection> = [
  {
    readouts: [
      {
        label: "Antialiasing",
        read: (applied, requested) =>
          toLevelAppliedChange(
            describeRenderAntialiasing(applied.antialiasing),
            describeRenderAntialiasing(requested.features.antialiasing)
          ),
      },
      {
        label: RENDER_UPSCALING_SETTINGS.isRayReconstructed.label,
        read: (applied, requested) =>
          applied.antialiasing === ERenderAntialiasing.DLSS
            ? toLevelAppliedChange(
                applied.isRayReconstructed ? "On" : "Off",
                requested.output.upscaling.isRayReconstructed ? "On" : "Off"
              )
            : null,
      },
      {
        label: RENDER_UPSCALING_SETTINGS.scale.label,
        read: (applied, requested) =>
          toLevelAppliedChange(
            describeRenderScale(applied.renderScale),
            describeRenderScale(requested.output.upscaling.scale)
          ),
      },
      {
        label: "Bloom",
        read: (applied, requested) =>
          applied.bloom
            ? toLevelAppliedChange(
                describeBloomMode(applied.bloom),
                describeBloomMode(requested.features.enhancedBloom.mode)
              )
            : "Off",
      },
      {
        label: "Sky debanding",
        read: (applied) => (applied.isSkyDebanded ? "Smoothed" : "The sky as drawn"),
      },
    ],
    title: "Applied image",
  },
  {
    readouts: [
      {
        label: "Ambient occlusion",
        read: (applied) =>
          applied.ambientOcclusion
            ? [
                applied.ambientOcclusionMethod
                  ? describeRenderAmbientOcclusionMethod(applied.ambientOcclusionMethod)
                  : null,
                describeRenderAmbientOcclusionSearch(applied.ambientOcclusion),
              ]
                .filter((it): it is string => it !== null)
                .join(", ")
            : "Off",
      },
      {
        label: "Indirect light",
        read: (applied) =>
          applied.indirectLight
            ? `${formatIndirectLightIntensity(applied.indirectLight.intensity ?? 0)} · ` +
              (applied.indirectLight.isShared ? "in VBAO's search" : "its own search")
            : "Off",
      },
      {
        label: "Reflections",
        read: (applied) =>
          applied.reflections
            ? `${formatReflectionIntensity(applied.reflections.intensity ?? 0)} · ` +
              `${describeRenderTraceSize(applied.reflections.traceSize).toLowerCase()} size · ` +
              `${formatCount(applied.reflections.steps)} steps`
            : "The cube alone",
      },
      {
        label: "Lamps",
        read: (applied) =>
          applied.lights
            ? [
                applied.lights.isShadowed
                  ? `shadowed, ${describeRenderLightShadowFilter(applied.lights.shadowFilter).toLowerCase()} filter`
                  : "unshadowed",
                applied.lights.isLevelLights ? "with the level's own" : null,
              ]
                .filter((it): it is string => it !== null)
                .join(", ")
            : "Off",
      },
      {
        label: "Sun shadows",
        read: (applied, requested) => {
          const shadows = applied.shadows;
          const asked = requested.features.shadows;

          return shadows
            ? `${toLevelAppliedChange(formatCascades(shadows.cascades), formatCascades(asked.cascades))} · ` +
                `${toLevelAppliedChange(String(shadows.resolution), String(asked.resolution))} texels · ` +
                toLevelAppliedChange(formatShadowFilter(shadows.filter), formatShadowFilter(asked.filter)).toLowerCase()
            : "Off";
        },
      },
    ],
    title: "Applied lighting",
  },
  {
    readouts: [
      {
        label: "Grass",
        read: ({ grass }) =>
          grass
            ? `${formatGrassRadius(grass.radius ?? 0)} · ${formatGrassDensity(grass.density ?? 0)} · ` +
              (grass.tufts < grass.wanted
                ? `${formatCount(grass.tufts)} of ${formatCount(grass.wanted)} tufts, capped by a GPU buffer limit`
                : `${formatCount(grass.wanted)} tufts`)
            : "Off",
      },
      { label: "Water", read: (applied) => (applied.isWater ? "Drawn" : "Off") },
      {
        label: "Height fog",
        read: ({ heightFog }) =>
          heightFog
            ? `${formatFogHeight(heightFog.height ?? 0)} · ${formatFogDensity(heightFog.density ?? 0)} · ` +
              (heightFog.scattering ? `${formatFogShare(heightFog.scattering)} scattering` : "no scattering") +
              ((heightFog.indoorShare ?? 1) < 1 ? ` · ${formatFogShare(heightFog.indoorShare ?? 1)} indoors` : "")
            : "The engine's fog alone",
      },
    ],
    title: "Applied world",
  },
  {
    isShown: (applied) => applied.environment !== null,
    readouts: [
      {
        isMonospace: true,
        label: "Sun direction",
        read: (applied) => readEnvironment(applied, (it) => formatTriple(it.sunDirection)),
      },
      {
        isMonospace: true,
        label: "Sun colour",
        read: (applied) => readEnvironment(applied, (it) => formatTriple(it.sunColor)),
      },
      {
        isMonospace: true,
        label: "Ambient",
        read: (applied) => readEnvironment(applied, (it) => formatTriple(it.ambient)),
      },
      {
        isMonospace: true,
        label: "Hemisphere",
        read: (applied) => readEnvironment(applied, (it) => formatTriple(it.hemisphere)),
      },
      {
        label: "Fog",
        read: (applied) =>
          readEnvironment(applied, (it) =>
            it.fog
              ? `${formatNumber(it.fog.distance ?? 0, 0)} m · density ${formatNumber(it.fog.density ?? 0, 2)}`
              : "None"
          ),
      },
      { label: "Rain", read: (applied) => readEnvironment(applied, (it) => formatNumber(it.rainDensity ?? 0, 2)) },
      { label: "Tree sway", read: (applied) => readEnvironment(applied, (it) => formatNumber(it.treeSway ?? 0, 3)) },
      {
        label: "Water intensity",
        read: (applied) => readEnvironment(applied, (it) => formatNumber(it.waterIntensity ?? 0, 2)),
      },
    ],
    title: "Applied environment",
  },
];
