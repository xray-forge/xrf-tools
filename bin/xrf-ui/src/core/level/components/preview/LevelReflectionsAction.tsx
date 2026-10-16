import { default as FlipIcon } from "@mui/icons-material/Flip";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import {
  ERenderReflectionMode,
  RenderReflectionProbe,
  RenderReflectionQuality,
  RenderReflectionTraceQuality,
} from "@/core/ipc/types/xrf-renderer";
import { LevelGlassSettings } from "@/core/level/components/preview/LevelGlassSettings";
import { ILevelFeatureOptions, TLevelReflectionOptions } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderReflectionQuality,
  explainRenderReflectionMode,
  formatReflectionDistance,
  formatReflectionIntensity,
  formatReflectionSkyGain,
  RENDER_REFLECTION_LIMITS,
  RENDER_REFLECTION_PROBE_OPTIONS,
  RENDER_REFLECTION_QUALITY_OPTIONS,
  RENDER_REFLECTION_TRACE_QUALITY_OPTIONS,
} from "@/core/render/lib/features/render-reflection-choices";
import { TRenderGlassSettings, TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelReflectionsActionProps extends BaseComponentProps {
  /** The reflections as the view draws them. */
  value: TRenderReflectionSettings;
  /** The glass as the view draws it, set here beside what reflects. */
  glass: TRenderGlassSettings;
  /** What the view sets over the settings, of which the reflections' part is changed. */
  features: ILevelFeatureOptions;
  /** Whether the GPU drawing the view traces rays, which tracing what the screen misses needs. */
  isTraceable: boolean;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * What glossy and wet surfaces reflect: the engine's sky cube alone, or screen-space reflections of what the frame
 * shows where a traced ray meets it; how hard they trace, how strongly surfaces reflect, how far a ray goes and how
 * fine the cube of the surroundings a ray meeting nothing reads is; and how glass is drawn.
 */
export function LevelReflectionsAction({
  "data-testid": dataTestId = "level-reflections-action",
  id,
  className,
  value,
  glass,
  features,
  isTraceable,
  onChange,
}: ILevelReflectionsActionProps): ReactElement {
  const isEnhanced: boolean = value.mode === ERenderReflectionMode.ENHANCED;

  const set = useCallback(
    (part: Partial<TLevelReflectionOptions>): void =>
      onChange({ ...features, reflections: { ...features.reflections, ...part } }),
    [features, onChange]
  );

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
      toggleLabel={"Trace reflections"}
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

          <CheckboxFormRow
            label={"Reflect grass"}
            isChecked={value.isGrassReflected}
            onChange={(isGrassReflected: boolean) => set({ isGrassReflected })}
          />

          <CheckboxFormRow
            label={"Trace what the screen misses"}
            description={isTraceable ? undefined : "Needs a GPU that traces rays"}
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
        </>
      ) : null}

      <Button size={"small"} onClick={() => onChange({ ...features, reflections: {} })}>
        Back to the settings for the reflections
      </Button>

      <LevelGlassSettings value={glass} features={features} onChange={onChange} />
    </EditorPopoverToggle>
  );
}
