import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  ERenderAntialiasing,
  ERenderReflectionMode,
  ERenderWaterMode,
  RenderReflectionProbe,
  RenderTraceSize,
} from "@/core/ipc/types/xrf-renderer";
import {
  explainRenderReflectionMode,
  formatReflectionDistance,
  formatReflectionIntensity,
  formatReflectionSkyGain,
  formatReflectionSteps,
  formatReflectionTraceInterval,
  formatReflectionTraceRoughness,
  RENDER_REFLECTION_LIMITS,
  RENDER_REFLECTION_PROBE_OPTIONS,
} from "@/core/render/lib/features/render-reflection-choices";
import { RENDER_TRACE_SIZE_OPTIONS } from "@/core/render/lib/features/render-trace-size-choices";
import { IRenderFeatureSettings, TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/**
 * What the frame reflects: the surfaces, the sky's cube alone or traced over the screen; the enhanced water's own
 * reflection of the screen; and the ray trace both take what the screen misses from.
 */
export function SettingsRendererReflections(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const reflections: TRenderReflectionSettings = features.reflections;
  const isSurfaces: boolean = reflections.mode === ERenderReflectionMode.ENHANCED;
  const isWaterEnhanced: boolean = features.water.mode === ERenderWaterMode.ENHANCED;
  // Ray reconstruction takes every pixel's own ray each frame, so the surfaces trace at full size every frame.
  const isRayReconstructed: boolean =
    features.antialiasing === ERenderAntialiasing.DLSS && features.upscaling.isRayReconstructed;

  const onSet = useRendererOverride("reflections");

  return (
    <>
      <DetailSection title={"Reflections: surfaces"} description={"The ground, puddles, and wet and glossy surfaces."}>
        <div className={"mt-4 flex flex-col gap-6"}>
          <CheckboxFormRow
            label={"Screen-space reflections"}
            description={explainRenderReflectionMode(reflections.mode)}
            isChecked={isSurfaces}
            onChange={(isOn: boolean) =>
              onSet({ mode: isOn ? ERenderReflectionMode.ENHANCED : ERenderReflectionMode.ENGINE })
            }
          />

          <ChoiceFormRow
            label={"Trace size"}
            description={
              isRayReconstructed
                ? "Full size every frame under ray reconstruction, which takes each pixel's own ray."
                : "Half traces a ray for every four pixels; full, one for each."
            }
            options={RENDER_TRACE_SIZE_OPTIONS}
            value={reflections.traceSize}
            isDisabled={!isSurfaces || isRayReconstructed}
            onChange={(traceSize: RenderTraceSize) => onSet({ traceSize })}
          />

          <SliderFormRow
            label={"Steps"}
            description={"How many cells of the depth a ray walks at most; a rough surface's ray walks half."}
            value={reflections.steps}
            {...RENDER_REFLECTION_LIMITS.steps}
            isDisabled={!isSurfaces}
            format={formatReflectionSteps}
            onChange={(steps: number) => onSet({ steps })}
          />

          <ChoiceFormRow
            label={"Surroundings"}
            description={
              "How fine the cube is that a ray meeting nothing on the screen reads; finer takes more memory."
            }
            options={RENDER_REFLECTION_PROBE_OPTIONS}
            value={reflections.probe}
            isDisabled={!isSurfaces}
            onChange={(probe: RenderReflectionProbe) => onSet({ probe })}
          />

          <SliderFormRow
            label={"Intensity"}
            description={"How much surfaces reflect: 1 as their material does."}
            value={reflections.intensity}
            {...RENDER_REFLECTION_LIMITS.intensity}
            isDisabled={!isSurfaces}
            format={formatReflectionIntensity}
            onChange={(intensity: number) => onSet({ intensity })}
          />

          <SliderFormRow
            label={"Distance"}
            description={"Metres a ray goes at most, the water's traced rays too."}
            value={reflections.distance}
            {...RENDER_REFLECTION_LIMITS.distance}
            isDisabled={!isSurfaces}
            format={formatReflectionDistance}
            onChange={(distance: number) => onSet({ distance })}
          />

          <SliderFormRow
            label={"Wet reach"}
            description={"Metres out to which the rain wets the ground for the rays to trace; the game wets 25 m."}
            value={reflections.reach}
            {...RENDER_REFLECTION_LIMITS.reach}
            isDisabled={!isSurfaces}
            format={formatReflectionDistance}
            onChange={(reach: number) => onSet({ reach })}
          />

          <SliderFormRow
            label={"Sky light"}
            description={"How bright the sky's highlights are in the reflections: 1 as the sky shows them."}
            value={reflections.skyGain}
            {...RENDER_REFLECTION_LIMITS.skyGain}
            isDisabled={!isSurfaces}
            format={formatReflectionSkyGain}
            onChange={(skyGain: number) => onSet({ skyGain })}
          />

          <SliderFormRow
            label={"Trace roughness"}
            description={
              "How rough a surface may be and still be traced; rougher ones reflect the sky. 0.4 traces less."
            }
            value={reflections.traceRoughness}
            {...RENDER_REFLECTION_LIMITS.traceRoughness}
            isDisabled={!isSurfaces}
            format={formatReflectionTraceRoughness}
            onChange={(traceRoughness: number) => onSet({ traceRoughness })}
          />

          <CheckboxFormRow
            label={"Trace dry surfaces"}
            description={
              "Traces a dry level's surfaces that show little of what they reflect, a dielectric's 4%; off, only " +
              "metal and glass-like surfaces trace until the rain wets the rest, which reflect the sky's cube."
            }
            isChecked={reflections.isDryTraced}
            isDisabled={!isSurfaces}
            onChange={(isDryTraced: boolean) => onSet({ isDryTraced })}
          />

          <CheckboxFormRow
            label={"Ray budgets"}
            description={
              "Traces fewer rays where a reflection is dim or rough, sharing them in each quad of pixels; saves most " +
              "at full size and under ray reconstruction, where every pixel would trace."
            }
            isChecked={reflections.isRayBudgeted}
            isDisabled={!isSurfaces}
            onChange={(isRayBudgeted: boolean) => onSet({ isRayBudgeted })}
          />

          <CheckboxFormRow
            label={"Reflect grass"}
            description={"Grass stands in the reflections and stops the rays behind it."}
            isChecked={reflections.isGrassReflected}
            isDisabled={!isSurfaces}
            onChange={(isGrassReflected: boolean) => onSet({ isGrassReflected })}
          />
        </div>
      </DetailSection>

      <DetailSection title={"Reflections: water"} description={"Rivers and lakes, as the enhanced water draws them."}>
        <div className={"mt-4 flex flex-col gap-6"}>
          <CheckboxFormRow
            label={"Water reflects the screen"}
            description={
              isWaterEnhanced
                ? "Off, the water reflects the sky's cube alone and keeps the rest of its look."
                : "The game's water reflects the sky's cube; set the water to Enhanced."
            }
            isChecked={isWaterEnhanced && reflections.isWaterReflected}
            isDisabled={!isWaterEnhanced}
            onChange={(isWaterReflected: boolean) => onSet({ isWaterReflected })}
          />

          <SliderFormRow
            label={"March steps"}
            description={"How many steps the water's reflection marches over the screen."}
            value={reflections.waterSteps}
            {...RENDER_REFLECTION_LIMITS.waterSteps}
            isDisabled={!isWaterEnhanced || !reflections.isWaterReflected}
            format={formatReflectionSteps}
            onChange={(waterSteps: number) => onSet({ waterSteps })}
          />
        </div>
      </DetailSection>

      <DetailSection
        title={"Reflections: ray tracing"}
        description={"What the surfaces' and the water's rays miss on the screen, traced against the level."}
      >
        <div className={"mt-4 flex flex-col gap-6"}>
          <CheckboxFormRow
            label={"Ray-traced reflections"}
            description={
              "Traces what the screen misses against the level's geometry on a GPU that traces rays, for the " +
              "surfaces and the water alike; elsewhere they reflect the sky's cube."
            }
            isChecked={reflections.isTraced}
            onChange={(isTraced: boolean) => onSet({ isTraced })}
          />

          <SliderFormRow
            label={"Re-trace"}
            description={
              isRayReconstructed
                ? "How often a missed pixel traces again, the last answer held between; the surfaces trace every " +
                  "frame under ray reconstruction."
                : "How often a missed pixel traces again, the last answer held between."
            }
            value={reflections.traceInterval}
            {...RENDER_REFLECTION_LIMITS.traceInterval}
            isDisabled={!reflections.isTraced}
            format={formatReflectionTraceInterval}
            onChange={(traceInterval: number) => onSet({ traceInterval })}
          />
        </div>
      </DetailSection>
    </>
  );
}
