import { default as GradientIcon } from "@mui/icons-material/Gradient";
import { Button } from "@mui/material";
import { ERendererAmbientOcclusionQuality, IRendererAmbientOcclusionSettings } from "@xrf/renderer";
import { ReactElement, useCallback } from "react";

import { ILevelFeatureOptions, TLevelAmbientOcclusionOptions } from "@/core/level/lib/features/level-feature-options";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderAmbientOcclusionQuality,
  RENDER_AMBIENT_OCCLUSION_LIMITS,
} from "@/core/render/lib/features/render-feature-choices";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";

const QUALITY_OPTIONS = Object.values(ERendererAmbientOcclusionQuality).map(
  (value: ERendererAmbientOcclusionQuality) => ({ label: describeRenderAmbientOcclusionQuality(value), value })
);

interface ILevelAmbientOcclusionActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The occlusion the view is drawn with: the settings', with the view's own values over them. */
  occlusion: IRendererAmbientOcclusionSettings;
  /** Whether the renderer's settings draw it at all, which this view can only narrow. */
  isAvailable?: boolean;
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * Whether this view darkens creases and corners by the screen's ambient occlusion, and how far, how dark, how fine.
 */
export function LevelAmbientOcclusionAction({
  "data-testid": dataTestId = "level-ambient-occlusion-action",
  id,
  className,
  isOn,
  occlusion,
  isAvailable = true,
  features,
  onToggle,
  onChange,
}: ILevelAmbientOcclusionActionProps): ReactElement {
  const set = useCallback(
    (part: Partial<TLevelAmbientOcclusionOptions>) =>
      onChange({ ...features, ambientOcclusion: { ...features.ambientOcclusion, ...part } }),
    [features, onChange]
  );

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Ambient occlusion"}
      description={
        !isAvailable
          ? "Ambient occlusion is off in Settings, under Rendering"
          : isOn
            ? `Ambient occlusion over ${formatNumber(occlusion.radius, 2)} m, ` +
              `${describeRenderAmbientOcclusionQuality(occlusion.quality).toLowerCase()} quality`
            : "Ambient occlusion off, only the baked occlusion shades"
      }
      icon={<GradientIcon />}
      isOn={isOn && isAvailable}
      isDisabled={!isAvailable}
      toggleLabel={"Darken creases and corners"}
      onToggle={onToggle}
    >
      <RenderValueChoice
        label={"Quality"}
        options={QUALITY_OPTIONS}
        value={occlusion.quality}
        onChange={(quality: ERendererAmbientOcclusionQuality) => set({ quality })}
      />

      <RenderValueSlider
        label={"Radius"}
        value={occlusion.radius}
        {...RENDER_AMBIENT_OCCLUSION_LIMITS.radius}
        format={(value: number) => `${formatNumber(value, 2)} m`}
        onChange={(radius: number) => set({ radius })}
      />

      <RenderValueSlider
        label={"Strength"}
        value={occlusion.strength}
        {...RENDER_AMBIENT_OCCLUSION_LIMITS.strength}
        format={(value: number) => formatNumber(value, 1)}
        onChange={(strength: number) => set({ strength })}
      />

      <Button size={"small"} onClick={() => onChange({ ...features, ambientOcclusion: {} })}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
