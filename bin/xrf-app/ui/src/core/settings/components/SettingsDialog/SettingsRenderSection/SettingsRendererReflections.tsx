import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import {
  ERenderAntialiasing,
  ERenderReflectionMode,
  RenderReflectionProbe,
  RenderTraceSize,
} from "@xrf/ipc/types/xrf-renderer";
import { CheckboxFormRow } from "@xrf/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@xrf/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@xrf/ui/form/SliderFormRow";
import { DetailSection } from "@xrf/ui/layout/DetailSection";
import { ReactElement } from "react";

import {
  explainRenderReflectionMode,
  formatReflectionDistance,
  formatReflectionIntensity,
  formatReflectionSkyGain,
  formatReflectionTraceRoughness,
  RENDER_REFLECTION_LIMITS,
  RENDER_REFLECTION_PROBE_OPTIONS,
} from "@/core/render/lib/features/render-reflection-choices";
import { RENDER_TRACE_SIZE_OPTIONS } from "@/core/render/lib/features/render-trace-size-choices";
import { IRenderFeatureSettings, TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings.service";

import { useRendererOverride } from "./use-renderer-override";

/** What the ground, puddles and wet and glossy surfaces reflect: the sky's cube alone, or traced over the screen. */
export function SettingsRendererReflections(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const features: IRenderFeatureSettings = renderSettingsService.rendererFeatures;
  const reflections: TRenderReflectionSettings = features.reflections;
  const isSurfaces: boolean = reflections.mode === ERenderReflectionMode.ENHANCED;
  // Ray reconstruction takes every pixel's own ray each frame, so the surfaces trace at full size every frame.
  const isRayReconstructed: boolean =
    features.antialiasing === ERenderAntialiasing.DLSS && features.upscaling.isRayReconstructed;

  const onSet = useRendererOverride("reflections");

  return (
    <DetailSection title={"Reflections"} description={"The ground, puddles, and wet and glossy surfaces."}>
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
          format={formatCount}
          onChange={(steps: number) => onSet({ steps })}
        />

        <ChoiceFormRow
          label={"Surroundings"}
          description={"How fine the cube is that a ray meeting nothing on the screen reads; finer takes more memory."}
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
          description={"How rough a surface may be and still be traced; rougher ones reflect the sky. 0.4 traces less."}
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
  );
}
