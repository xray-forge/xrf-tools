import { default as FlipIcon } from "@mui/icons-material/Flip";
import { Button } from "@mui/material";
import { useInjection } from "@wirestate/react";
import { formatCount } from "@xrf/format";
import { ReactElement } from "react";

import {
  ERenderReflectionMode,
  ERenderTraceSize,
  ERenderWaterMode,
  RenderReflectionProbe,
  RenderTraceSize,
} from "@/core/ipc/types/xrf-renderer";
import { LevelGlassSettings } from "@/core/level/components/preview/toolbar/lighting/LevelGlassSettings";
import { ILevelFeature, useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { LevelRenderService } from "@/core/level/services/level-render.service";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderReflectionMode,
  formatReflectionDistance,
  formatReflectionIntensity,
  formatReflectionSkyGain,
  formatReflectionTraceInterval,
  formatReflectionTraceRoughness,
  RENDER_REFLECTION_LIMITS,
  RENDER_REFLECTION_PROBE_OPTIONS,
} from "@/core/render/lib/features/render-reflection-choices";
import { RENDER_TRACE_SIZE_OPTIONS } from "@/core/render/lib/features/render-trace-size-choices";
import { TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@/core/shell/editor/EditorPopoverGroup";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * What the view reflects, in three sections: the surfaces (the ground, puddles, wet and glossy surfaces), the
 * enhanced water's own reflection of the screen, and the ray trace both take what the screen misses from; then how
 * glass is drawn.
 */
export function LevelReflectionsAction({
  "data-testid": dataTestId = "level-reflections-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const renderService: LevelRenderService = useInjection(LevelRenderService);
  const { settings, features } = useLevelRenderFeatures();
  const { set, reset } = useLevelFeatureOverride("reflections");
  const water: ILevelFeature<"water"> = useLevelFeature("water");

  const value: TRenderReflectionSettings = toLevelModeFeature("reflections", settings, features);
  const isTraceable: boolean = renderService.isTraceable;
  // Under ray reconstruction the surfaces trace every pixel every frame, at the frame's size.
  const isRayReconstructed: boolean = renderService.isRayReconstructed;
  const isSurfaces: boolean = value.mode === ERenderReflectionMode.ENHANCED;
  const isWaterEnhanced: boolean = water.state.value.mode === ERenderWaterMode.ENHANCED;
  const isWater: boolean = isWaterEnhanced && value.isWaterReflected;
  const isTraced: boolean = isTraceable && value.isTraced;

  return (
    <EditorPopoverGroup
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Reflections"}
      description={[
        isSurfaces ? "Surfaces traced over the screen" : "Surfaces reflecting the sky's cube",
        isWater ? "the water reflecting the screen" : null,
        isTraced ? "misses ray-traced" : null,
      ]
        .filter(Boolean)
        .join(", ")}
      icon={<FlipIcon />}
      isActive={isSurfaces || isWater || isTraced}
    >
      <EditorPopoverGroupSection
        label={"Surfaces"}
        description={explainRenderReflectionMode(value.mode)}
        isOn={isSurfaces}
        onToggle={() => set({ mode: isSurfaces ? ERenderReflectionMode.ENGINE : ERenderReflectionMode.ENHANCED })}
      >
        {isSurfaces ? (
          <>
            {isRayReconstructed ? (
              <p className={"text-xs text-text-secondary"}>
                Traced at full size every frame: ray reconstruction takes each pixel&apos;s own ray.
              </p>
            ) : null}

            <RenderValueChoice
              label={"Trace size"}
              options={RENDER_TRACE_SIZE_OPTIONS}
              value={isRayReconstructed ? ERenderTraceSize.FULL : value.traceSize}
              isDisabled={isRayReconstructed}
              onChange={(traceSize: RenderTraceSize) => set({ traceSize })}
            />

            <RenderValueSlider
              label={"Steps"}
              value={value.steps}
              {...RENDER_REFLECTION_LIMITS.steps}
              format={formatCount}
              onChange={(steps: number) => set({ steps })}
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
              label={"Trace dry surfaces"}
              description={"Off, a dry level traces only metal and glass-like surfaces until the rain wets it"}
              isChecked={value.isDryTraced}
              onChange={(isDryTraced: boolean) => set({ isDryTraced })}
            />

            <CheckboxFormRow
              label={"Ray budgets"}
              description={
                "Fewer rays where a reflection is dim or rough, shared in each quad; saves most at full size and " +
                "under ray reconstruction"
              }
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
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Water"}
        description={
          isWaterEnhanced
            ? "Rivers and lakes reflect the screen; off, the sky's cube alone"
            : "The game's water reflects the sky's cube: Water, Enhanced"
        }
        isOn={isWater}
        isDisabled={!isWaterEnhanced}
        onToggle={() => set({ isWaterReflected: !value.isWaterReflected })}
      >
        {isWater ? (
          <RenderValueSlider
            label={"March steps"}
            value={value.waterSteps}
            {...RENDER_REFLECTION_LIMITS.waterSteps}
            format={formatCount}
            onChange={(waterSteps: number) => set({ waterSteps })}
          />
        ) : null}
      </EditorPopoverGroupSection>

      <EditorPopoverGroupSection
        label={"Ray tracing"}
        description={
          isTraceable
            ? "Traces what the surfaces' and the water's rays miss on the screen against the level"
            : "Needs a GPU that traces rays"
        }
        isOn={isTraced}
        isDisabled={!isTraceable}
        onToggle={() => set({ isTraced: !value.isTraced })}
      >
        {isTraced ? (
          <RenderValueSlider
            label={"Re-trace"}
            value={value.traceInterval}
            {...RENDER_REFLECTION_LIMITS.traceInterval}
            format={formatReflectionTraceInterval}
            onChange={(traceInterval: number) => set({ traceInterval })}
          />
        ) : null}

        {isTraced && isRayReconstructed && isSurfaces ? (
          <p className={"text-xs text-text-secondary"}>The surfaces re-trace every frame under ray reconstruction.</p>
        ) : null}
      </EditorPopoverGroupSection>

      <Button size={"small"} aria-label={"Reset the reflections to the preset"} onClick={reset}>
        Reset to the preset
      </Button>

      <LevelGlassSettings className={"border-t border-divider pt-2"} />
    </EditorPopoverGroup>
  );
}
