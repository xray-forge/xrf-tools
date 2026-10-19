import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  ERenderReflectionMode,
  RenderReflectionProbe,
  RenderReflectionQuality,
  RenderReflectionTraceQuality,
} from "@/core/ipc/types/xrf-renderer";
import {
  explainRenderReflectionMode,
  formatReflectionDistance,
  formatReflectionIntensity,
  formatReflectionSkyGain,
  formatReflectionTraceRoughness,
  RENDER_REFLECTION_LIMITS,
  RENDER_REFLECTION_PROBE_OPTIONS,
  RENDER_REFLECTION_QUALITY_OPTIONS,
  RENDER_REFLECTION_TRACE_QUALITY_OPTIONS,
} from "@/core/render/lib/features/render-reflection-choices";
import { TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { RenderSettingsService } from "@/core/render/services/render-settings";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { ChoiceFormRow } from "@/core/ui/form/ChoiceFormRow";
import { SliderFormRow } from "@/core/ui/form/SliderFormRow";
import { DetailSection } from "@/core/ui/layout/DetailSection";

import { useRendererOverride } from "./use-renderer-override";

/** What glossy and wet surfaces reflect: the sky's cube alone, or what rays traced over the screen meet. */
export function SettingsRendererReflections(): ReactElement {
  const renderSettingsService: RenderSettingsService = useInjection(RenderSettingsService);

  const reflections: TRenderReflectionSettings = renderSettingsService.rendererFeatures.reflections;
  const isEnhanced: boolean = reflections.mode === ERenderReflectionMode.ENHANCED;

  const onSet = useRendererOverride("reflections");

  return (
    <DetailSection title={"Reflections"} description={"What glossy and wet surfaces reflect."}>
      <div className={"mt-4 flex flex-col gap-6"}>
        <CheckboxFormRow
          label={"Screen-space reflections"}
          description={explainRenderReflectionMode(reflections.mode)}
          isChecked={isEnhanced}
          onChange={(isOn: boolean) =>
            onSet({ mode: isOn ? ERenderReflectionMode.ENHANCED : ERenderReflectionMode.ENGINE })
          }
        />

        <ChoiceFormRow
          label={"Quality"}
          description={"The size rays are traced at and how far each goes: half size up to High, full at Ultra."}
          options={RENDER_REFLECTION_QUALITY_OPTIONS}
          value={reflections.quality}
          isDisabled={!isEnhanced}
          onChange={(quality: RenderReflectionQuality) => onSet({ quality })}
        />

        <ChoiceFormRow
          label={"Surroundings"}
          description={"How fine the cube is that a ray meeting nothing on the screen reads; finer takes more memory."}
          options={RENDER_REFLECTION_PROBE_OPTIONS}
          value={reflections.probe}
          isDisabled={!isEnhanced}
          onChange={(probe: RenderReflectionProbe) => onSet({ probe })}
        />

        <SliderFormRow
          label={"Intensity"}
          description={"How much surfaces reflect: 1 as their material does."}
          value={reflections.intensity}
          {...RENDER_REFLECTION_LIMITS.intensity}
          isDisabled={!isEnhanced}
          format={formatReflectionIntensity}
          onChange={(intensity: number) => onSet({ intensity })}
        />

        <SliderFormRow
          label={"Distance"}
          description={"Metres a ray is traced at most."}
          value={reflections.distance}
          {...RENDER_REFLECTION_LIMITS.distance}
          isDisabled={!isEnhanced}
          format={formatReflectionDistance}
          onChange={(distance: number) => onSet({ distance })}
        />

        <SliderFormRow
          label={"Wet reach"}
          description={"Metres out to which the rain wets the ground for the rays to trace; the game wets 25 m."}
          value={reflections.reach}
          {...RENDER_REFLECTION_LIMITS.reach}
          isDisabled={!isEnhanced}
          format={formatReflectionDistance}
          onChange={(reach: number) => onSet({ reach })}
        />

        <SliderFormRow
          label={"Sky light"}
          description={"How bright the sky's highlights are in the reflections: 1 as the sky shows them."}
          value={reflections.skyGain}
          {...RENDER_REFLECTION_LIMITS.skyGain}
          isDisabled={!isEnhanced}
          format={formatReflectionSkyGain}
          onChange={(skyGain: number) => onSet({ skyGain })}
        />

        <CheckboxFormRow
          label={"Reflect grass"}
          description={"Grass stands in the reflections and stops the rays behind it."}
          isChecked={reflections.isGrassReflected}
          isDisabled={!isEnhanced}
          onChange={(isGrassReflected: boolean) => onSet({ isGrassReflected })}
        />

        <CheckboxFormRow
          label={"Ray-traced reflections"}
          description={
            "Traces what the screen misses against the level's geometry, on a GPU that traces rays: the water's " +
            "reflection always, the surfaces' with screen-space reflections."
          }
          isChecked={reflections.isTraced}
          onChange={(isTraced: boolean) => onSet({ isTraced })}
        />

        <ChoiceFormRow
          label={"Trace quality"}
          description={"How often a pixel is traced: once in eight frames at Low, every frame at Ultra."}
          options={RENDER_REFLECTION_TRACE_QUALITY_OPTIONS}
          value={reflections.traceQuality}
          isDisabled={!reflections.isTraced}
          onChange={(traceQuality: RenderReflectionTraceQuality) => onSet({ traceQuality })}
        />

        <SliderFormRow
          label={"Trace roughness"}
          description={"How rough a surface may be and still be traced; rougher ones reflect the sky. 0.4 traces less."}
          value={reflections.traceRoughness}
          {...RENDER_REFLECTION_LIMITS.traceRoughness}
          isDisabled={!isEnhanced}
          format={formatReflectionTraceRoughness}
          onChange={(traceRoughness: number) => onSet({ traceRoughness })}
        />

        <CheckboxFormRow
          label={"Ray budgets"}
          description={
            "Traces fewer rays where a reflection is dim or rough, sharing them between neighbouring pixels."
          }
          isChecked={reflections.isRayBudgeted}
          isDisabled={!isEnhanced}
          onChange={(isRayBudgeted: boolean) => onSet({ isRayBudgeted })}
        />
      </div>
    </DetailSection>
  );
}
