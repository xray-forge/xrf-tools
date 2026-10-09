import { default as OpacityIcon } from "@mui/icons-material/Opacity";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderRainMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureOptions, TLevelRainOptions } from "@/core/level/lib/features";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  explainRenderRainMode,
  formatGrassRadius,
  formatRainShare,
  RENDER_RAIN_LIMITS,
} from "@/core/render/lib/features";
import { TRenderRainSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelWetSurfacesActionProps extends BaseComponentProps {
  /** The rain's wetting as the view draws it. */
  value: TRenderRainSettings;
  /** What the view sets over the settings, of which the rain's part is changed. */
  features: ILevelFeatureOptions;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * How rain wets the surfaces: the engine's splashes near the camera, or wet surfaces that build up and dry with
 * puddles on flat ground; how much of the ground puddles cover, how much they reflect and how strongly rain ripples.
 */
export function LevelWetSurfacesAction({
  "data-testid": dataTestId = "level-wet-surfaces-action",
  id,
  className,
  value,
  features,
  onChange,
}: ILevelWetSurfacesActionProps): ReactElement {
  const isEnhanced: boolean = value.mode === ERenderRainMode.ENHANCED;

  const set = useCallback(
    (part: Partial<TLevelRainOptions>): void => onChange({ ...features, rain: { ...features.rain, ...part } }),
    [features, onChange]
  );

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Wet surfaces"}
      description={
        isEnhanced
          ? `Wet surfaces and puddles, ${formatRainShare(value.puddles)} of flat ground to ` +
            `${formatGrassRadius(value.distance)}, ${formatRainShare(value.reflectivity)} reflective`
          : "The engine's splashes"
      }
      icon={<OpacityIcon />}
      isOn={isEnhanced}
      toggleLabel={"Wet surfaces and puddles"}
      onToggle={() => set({ mode: isEnhanced ? ERenderRainMode.ENGINE : ERenderRainMode.ENHANCED })}
    >
      <p className={"text-xs text-text-secondary"}>{explainRenderRainMode(value.mode)}</p>

      {isEnhanced ? (
        <>
          <RenderValueSlider
            label={"Puddles"}
            value={value.puddles}
            {...RENDER_RAIN_LIMITS.puddles}
            format={formatRainShare}
            onChange={(puddles: number) => set({ puddles })}
          />

          <RenderValueSlider
            label={"Reflectivity"}
            value={value.reflectivity}
            {...RENDER_RAIN_LIMITS.reflectivity}
            format={formatRainShare}
            onChange={(reflectivity: number) => set({ reflectivity })}
          />

          <RenderValueSlider
            label={"Distance"}
            value={value.distance}
            {...RENDER_RAIN_LIMITS.distance}
            format={formatGrassRadius}
            onChange={(distance: number) => set({ distance })}
          />

          <RenderValueSlider
            label={"Ripples"}
            value={value.ripples}
            {...RENDER_RAIN_LIMITS.ripples}
            format={formatRainShare}
            onChange={(ripples: number) => set({ ripples })}
          />
        </>
      ) : null}

      <Button size={"small"} onClick={() => onChange({ ...features, rain: {} })}>
        Back to the settings for the rain
      </Button>
    </EditorPopoverToggle>
  );
}
