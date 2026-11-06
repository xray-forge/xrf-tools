import { useInjection } from "@wirestate/react";
import {
  ERenderReflectionMode,
  ERenderTraceSize,
  ERenderWaterMode,
  RenderTraceSize,
} from "@xrf/ipc/types/xrf-renderer";
import { Optional } from "@xrf/types";
import { EditorPopoverGroup, EditorPopoverGroupSection } from "@xrf/ui/editor/popover-group";
import { EditorPopoverNote } from "@xrf/ui/editor/popover-row";
import { BaseComponentProps } from "@xrf/ui/element-types";
import { FlipIcon } from "@xrf/ui/icons/flip";
import { ReactElement } from "react";

import { LevelEnhancedFeatureSection } from "@/core/level/components/preview/toolbar/LevelEnhancedFeatureSection";
import { ILevelFeature, useLevelFeature } from "@/core/level/components/preview/toolbar/use-level-feature";
import { useLevelFeatureOverride } from "@/core/level/components/preview/toolbar/use-level-feature-override";
import {
  LEVEL_SETTINGS_RESET_LABEL,
  useLevelFeatureReset,
} from "@/core/level/components/preview/toolbar/use-level-feature-reset";
import { useLevelRenderFeatures } from "@/core/level/components/preview/toolbar/use-level-render-features";
import { toLevelModeFeature } from "@/core/level/lib/features/level-feature-options";
import { LevelRenderService } from "@/core/level/services/level-render";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderSettingEntries } from "@/core/render/components/settings/RenderSettingEntries";
import { RenderSettingEntry } from "@/core/render/components/settings/RenderSettingEntry";
import {
  explainRenderReflectionMode,
  RENDER_REFLECTION_SETTINGS,
} from "@/core/render/lib/features/tracing/render-reflection-choices";
import { RENDER_TRACE_SIZE_OPTIONS } from "@/core/render/lib/features/tracing/render-trace-size-choices";
import { TRenderReflectionSettings } from "@/core/render/lib/settings/choice/render-feature-settings";

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
  const { settings, features, drawn } = useLevelRenderFeatures();
  const { set } = useLevelFeatureOverride("reflections");
  // The surfaces', the water's and the trace's values are one group, set back together from the popover's heading.
  const onReset: Optional<() => void> = useLevelFeatureReset("reflections");
  const water: ILevelFeature<"water"> = useLevelFeature("water");

  const value: TRenderReflectionSettings = toLevelModeFeature("reflections", settings, features);
  const isTraceable: boolean = renderService.drawing.isTraceable;
  // Under ray reconstruction the surfaces trace every pixel every frame, at the frame's size.
  const isRayReconstructed: boolean = renderService.drawing.isRayReconstructed;
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
      resetLabel={LEVEL_SETTINGS_RESET_LABEL}
      onReset={onReset}
    >
      <EditorPopoverGroupSection
        label={"Surfaces"}
        note={explainRenderReflectionMode(value.mode)}
        isOn={isSurfaces}
        onToggle={() => set({ mode: isSurfaces ? ERenderReflectionMode.ENGINE : ERenderReflectionMode.ENHANCED })}
      >
        {isSurfaces && isRayReconstructed ? (
          <EditorPopoverNote>
            Traced at full size every frame: ray reconstruction takes each pixel&apos;s own ray.
          </EditorPopoverNote>
        ) : null}

        <RenderValueChoice
          label={RENDER_REFLECTION_SETTINGS.traceSize.label}
          options={RENDER_TRACE_SIZE_OPTIONS}
          value={isRayReconstructed ? ERenderTraceSize.FULL : value.traceSize}
          isDisabled={!isSurfaces || isRayReconstructed}
          onChange={(traceSize: RenderTraceSize) => set({ traceSize })}
        />

        <RenderSettingEntries
          settings={RENDER_REFLECTION_SETTINGS}
          names={[
            "steps",
            "probe",
            "intensity",
            "distance",
            "reach",
            "skyGain",
            "traceRoughness",
            "isDryTraced",
            "isRayBudgeted",
            "isGrassReflected",
          ]}
          group={value}
          features={drawn}
          onSet={set}
        />
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
        <RenderSettingEntry
          settings={RENDER_REFLECTION_SETTINGS}
          name={"waterSteps"}
          group={value}
          features={drawn}
          isDisabled={!isWater}
          onSet={set}
        />
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
        <RenderSettingEntry
          settings={RENDER_REFLECTION_SETTINGS}
          name={"traceInterval"}
          group={value}
          features={drawn}
          isDisabled={!isTraced}
          onSet={set}
        />

        <RenderSettingEntry
          settings={RENDER_REFLECTION_SETTINGS}
          name={"isWaterFullyTraced"}
          group={value}
          features={drawn}
          isDisabled={!isTraced || !isWater}
          onSet={set}
        />

        {isTraced && isRayReconstructed && isSurfaces ? (
          <EditorPopoverNote>The surfaces re-trace every frame under ray reconstruction.</EditorPopoverNote>
        ) : null}
      </EditorPopoverGroupSection>

      <LevelEnhancedFeatureSection feature={"glass"} />
    </EditorPopoverGroup>
  );
}
