import { Nullable } from "@xrf/types";
import { ReactElement } from "react";

import { RenderAppliedReport, RenderViewOptions } from "@/core/ipc/types/xrf-renderer";
import {
  describeRenderAmbientOcclusionQuality,
  describeRenderAntialiasing,
  describeRenderLightShadowFilter,
  describeRenderReflectionQuality,
  describeRenderScale,
  formatGrassDensity,
  formatGrassRadius,
  formatIndirectLightIntensity,
  formatReflectionIntensity,
  formatShadowFilter,
} from "@/core/render/lib/features/render-feature-choices";
import { EditorPanelProperty, EditorPanelSection } from "@/core/shell/editor/EditorPanel";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatCount, formatNumber } from "@/lib/format/number";

interface ILevelRendererAppliedSectionProps extends BaseComponentProps {
  /** What the renderer draws the level with, or null until it says. */
  applied: Nullable<RenderAppliedReport>;
  /** What the level asked it to draw with, which a changed value is told against. */
  requested: RenderViewOptions;
}

/** Three channels as the panel reads them. */
function formatTriple(values: ReadonlyArray<Nullable<number>>): string {
  return values.map((value: Nullable<number>) => formatNumber(value ?? 0, 2)).join("  ");
}

/** A value, and what was asked where the renderer drew something else. */
function toChanged(value: string, asked: string): string {
  return value === asked ? value : `${value} · asked ${asked}`;
}

/**
 * What the renderer actually draws the level with, which the settings and the toolbar only ask for: read-only, a value
 * the renderer changed saying what was asked.
 */
export function LevelRendererAppliedSection({
  "data-testid": dataTestId = "level-renderer-applied-section",
  id,
  className,
  applied,
  requested,
}: ILevelRendererAppliedSectionProps): ReactElement {
  if (!applied) {
    return (
      <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Applied"}>
        <EditorPanelProperty label={"Renderer"} value={"Not drawing yet"} />
      </EditorPanelSection>
    );
  }

  const { environment, grass, shadows } = applied;

  return (
    <>
      <EditorPanelSection data-testid={dataTestId} id={id} className={className} title={"Applied image"}>
        <EditorPanelProperty
          label={"Antialiasing"}
          value={toChanged(
            describeRenderAntialiasing(applied.antialiasing),
            describeRenderAntialiasing(requested.features.antialiasing)
          )}
        />
        <EditorPanelProperty
          label={"Render scale"}
          value={toChanged(
            describeRenderScale(applied.renderScale),
            describeRenderScale(requested.output.upscaling.scale)
          )}
        />
      </EditorPanelSection>

      <EditorPanelSection title={"Applied lighting"}>
        <EditorPanelProperty
          label={"Ambient occlusion"}
          value={applied.ambientOcclusion ? describeRenderAmbientOcclusionQuality(applied.ambientOcclusion) : "Off"}
        />
        <EditorPanelProperty
          label={"Indirect light"}
          value={
            applied.indirectLight
              ? `${formatIndirectLightIntensity(applied.indirectLight.intensity ?? 0)} · ` +
                (applied.indirectLight.isShared ? "in VBAO's search" : "its own search")
              : "Off"
          }
        />
        <EditorPanelProperty
          label={"Reflections"}
          value={
            applied.reflections
              ? `${formatReflectionIntensity(applied.reflections.intensity ?? 0)} · ` +
                `${describeRenderReflectionQuality(applied.reflections.quality).toLowerCase()} quality`
              : "The cube alone"
          }
        />
        <EditorPanelProperty
          label={"Lights"}
          value={
            applied.lights
              ? [
                  applied.lights.isShadowed
                    ? `shadowed, ${describeRenderLightShadowFilter(applied.lights.shadowFilter).toLowerCase()} filter`
                    : "unshadowed",
                  applied.lights.isLevelLights ? "with the level's own" : null,
                ]
                  .filter((it): it is string => it !== null)
                  .join(", ")
              : "Off"
          }
        />
        <EditorPanelProperty
          label={"Sun shadow"}
          value={
            shadows
              ? `${shadows.cascades.map((width) => formatNumber(width ?? 0, 0)).join(" / ")} m · ` +
                `${toChanged(String(shadows.resolution), String(requested.features.shadows.resolution))} texels · ` +
                formatShadowFilter(shadows.filter).toLowerCase()
              : "Off"
          }
        />
      </EditorPanelSection>

      <EditorPanelSection title={"Applied world"}>
        <EditorPanelProperty
          label={"Grass"}
          value={
            grass
              ? `${formatGrassRadius(grass.radius ?? 0)} · ${formatGrassDensity(grass.density ?? 0)} · ` +
                (grass.tufts < grass.wanted
                  ? `${formatCount(grass.tufts)} of ${formatCount(grass.wanted)} tufts, capped by a GPU buffer limit`
                  : `${formatCount(grass.wanted)} tufts`)
              : "Off"
          }
        />
        <EditorPanelProperty label={"Water"} value={applied.isWater ? "Drawn" : "Off"} />
      </EditorPanelSection>

      {environment ? (
        <EditorPanelSection title={"Applied environment"}>
          <EditorPanelProperty label={"Sun direction"} value={formatTriple(environment.sunDirection)} isMonospace />
          <EditorPanelProperty label={"Sun colour"} value={formatTriple(environment.sunColor)} isMonospace />
          <EditorPanelProperty label={"Ambient"} value={formatTriple(environment.ambient)} isMonospace />
          <EditorPanelProperty label={"Hemisphere"} value={formatTriple(environment.hemisphere)} isMonospace />
          <EditorPanelProperty
            label={"Fog"}
            value={
              environment.fog
                ? `${formatNumber(environment.fog.distance ?? 0, 0)} m · density ${formatNumber(environment.fog.density ?? 0, 2)}`
                : "None"
            }
          />
          <EditorPanelProperty label={"Rain"} value={formatNumber(environment.rainDensity ?? 0, 2)} />
          <EditorPanelProperty label={"Tree sway"} value={formatNumber(environment.treeSway ?? 0, 3)} />
          <EditorPanelProperty label={"Water intensity"} value={formatNumber(environment.waterIntensity ?? 0, 2)} />
        </EditorPanelSection>
      ) : null}
    </>
  );
}
