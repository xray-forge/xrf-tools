import { default as FlipIcon } from "@mui/icons-material/Flip";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { ReactElement } from "react";

import {
  ERenderReflectionMode,
  RenderReflectionProbe,
  RenderReflectionQuality,
  RenderReflectionTraceQuality,
} from "@/core/ipc/types/xrf-renderer";
import { LevelGlassSettings } from "@/core/level/components/preview/toolbar/lighting/LevelGlassSettings";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderReflectionQuality,
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
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What glossy and wet surfaces reflect: the engine's sky cube alone, or screen-space reflections of what the frame
 * shows where a traced ray meets it; whether what the screen misses is ray-traced on a GPU that traces rays, for the
 * surfaces and the water's own reflection alike, and how often; how hard the surfaces trace, how
 * strongly surfaces reflect, how far a ray goes, how rough a surface still traces, whether rays are budgeted by what
 * each pixel shows, and how fine the cube of the surroundings a ray meeting nothing reads is; and how glass is drawn.
 */
export function LevelReflectionsAction({
  "data-testid": dataTestId = "level-reflections-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("reflections");

  const value: TRenderReflectionSettings = toLevelModeFeature("reflections", settings, features);
  const isTraceable: boolean = renderService.isTraceable;
  const isEnhanced: boolean = value.mode === ERenderReflectionMode.ENHANCED;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Reflections"}
      description={
        isEnhanced
          ? `Screen-space reflections, ${describeRenderReflectionQuality(value.quality).toLowerCase()} quality, ` +
            `${formatReflectionIntensity(value.intensity)} over ${formatReflectionDistance(value.distance)}`
          : "The sky's cube alone"
      }
      icon={<FlipIcon />}
      isOn={isEnhanced}
      toggleLabel={"Screen-space reflections"}
      onToggle={() => set({ mode: isEnhanced ? ERenderReflectionMode.ENGINE : ERenderReflectionMode.ENHANCED })}
    >
      <p className={"text-xs text-text-secondary"}>{explainRenderReflectionMode(value.mode)}</p>

      {isEnhanced ? (
        <>
          <RenderValueChoice
            label={"Quality"}
            options={RENDER_REFLECTION_QUALITY_OPTIONS}
            value={value.quality}
            onChange={(quality: RenderReflectionQuality) => set({ quality })}
          />

          <RenderValueChoice
            label={"Surroundings"}
            options={RENDER_REFLECTION_PROBE_OPTIONS}
            value={value.probe}
            onChange={(probe: RenderReflectionProbe) => set({ probe })}
          />

          <RenderValueSlider
            label={"Intensity"}
            value={value.intensity}
            {...RENDER_REFLECTION_LIMITS.intensity}
            format={formatReflectionIntensity}
            onChange={(intensity: number) => set({ intensity })}
          />

          <RenderValueSlider
            label={"Distance"}
            value={value.distance}
            {...RENDER_REFLECTION_LIMITS.distance}
            format={formatReflectionDistance}
            onChange={(distance: number) => set({ distance })}
          />

          <RenderValueSlider
            label={"Wet reach"}
            value={value.reach}
            {...RENDER_REFLECTION_LIMITS.reach}
            format={formatReflectionDistance}
            onChange={(reach: number) => set({ reach })}
          />

          <RenderValueSlider
            label={"Sky light"}
            value={value.skyGain}
            {...RENDER_REFLECTION_LIMITS.skyGain}
            format={formatReflectionSkyGain}
            onChange={(skyGain: number) => set({ skyGain })}
          />

          <RenderValueSlider
            label={"Trace roughness"}
            value={value.traceRoughness}
            {...RENDER_REFLECTION_LIMITS.traceRoughness}
            format={formatReflectionTraceRoughness}
            onChange={(traceRoughness: number) => set({ traceRoughness })}
          />

          <CheckboxFormRow
            label={"Ray budgets"}
            description={"Fewer rays where a reflection is dim or rough, shared between neighbouring pixels"}
            isChecked={value.isRayBudgeted}
            onChange={(isRayBudgeted: boolean) => set({ isRayBudgeted })}
          />

          <CheckboxFormRow
            label={"Reflect grass"}
            isChecked={value.isGrassReflected}
            onChange={(isGrassReflected: boolean) => set({ isGrassReflected })}
          />
        </>
      ) : null}

      <CheckboxFormRow
        label={"Ray-traced reflections"}
        description={
          isTraceable
            ? "Traces what the screen misses: the water's reflection always, the surfaces' with screen-space reflections"
            : "Needs a GPU that traces rays"
        }
        isChecked={isTraceable && value.isTraced}
        isDisabled={!isTraceable}
        onChange={(isTraced: boolean) => set({ isTraced })}
      />

      {isTraceable && value.isTraced ? (
        <RenderValueChoice
          label={"Trace quality"}
          options={RENDER_REFLECTION_TRACE_QUALITY_OPTIONS}
          value={value.traceQuality}
          onChange={(traceQuality: RenderReflectionTraceQuality) => set({ traceQuality })}
        />
      ) : null}

      <Button size={"small"} aria-label={"Reset the reflections to the preset"} onClick={reset}>
        Reset to the preset
      </Button>

      <LevelGlassSettings />
    </EditorPopoverToggle>
  );
}
