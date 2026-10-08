import { default as FlipIcon } from "@mui/icons-material/Flip";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderReflectionMode, RenderReflectionQuality } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureOptions, TLevelReflectionOptions } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderReflectionQuality,
  explainRenderReflectionMode,
  formatReflectionDistance,
  formatReflectionIntensity,
  RENDER_REFLECTION_LIMITS,
  RENDER_REFLECTION_QUALITY_OPTIONS,
} from "@/core/render/lib/features";
import { TRenderReflectionSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelReflectionsActionProps extends BaseComponentProps {
  /** The reflections as the view draws them. */
  value: TRenderReflectionSettings;
  /** What the view sets over the settings, of which the reflections' part is changed. */
  features: ILevelFeatureOptions;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * What glossy and wet surfaces reflect: the engine's sky cube alone, or screen-space reflections of what the frame
 * shows where a traced ray meets it; how hard they trace, how much of the cube a hit replaces and how far a ray goes.
 */
export function LevelReflectionsAction({
  "data-testid": dataTestId = "level-reflections-action",
  id,
  className,
  value,
  features,
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
        </>
      ) : null}

      <Button size={"small"} onClick={() => onChange({ ...features, reflections: {} })}>
        Back to the settings for the reflections
      </Button>
    </EditorPopoverToggle>
  );
}
