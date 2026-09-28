import { default as GradientIcon } from "@mui/icons-material/Gradient";
import { Button } from "@mui/material";
import { ERendererAmbientOcclusionQuality, IRendererAmbientOcclusionSettings } from "@xrf/renderer";
import { ReactElement } from "react";

import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  describeRenderAmbientOcclusionQuality,
  formatOcclusionRadius,
  formatOcclusionStrength,
  RENDER_AMBIENT_OCCLUSION_LIMITS,
  RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS,
} from "@/core/render/lib/features";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";

/**
 * Whether this view darkens creases and corners by the screen's ambient occlusion, and how far, how dark, how fine.
 */
export function LevelAmbientOcclusionAction({
  "data-testid": dataTestId = "level-ambient-occlusion-action",
  id,
  className,
  isOn,
  state,
  features,
  onToggle,
  onChange,
}: ILevelFeatureActionProps<"ambientOcclusion">): ReactElement {
  const { set, reset } = useLevelFeatureOverride("ambientOcclusion", features, onChange);

  const occlusion: IRendererAmbientOcclusionSettings = state.value;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Ambient occlusion"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        label: "Ambient occlusion",
        off: "Ambient occlusion off, only the baked occlusion shades",
        on:
          `Ambient occlusion over ${formatOcclusionRadius(occlusion.radius)}, ` +
          `${describeRenderAmbientOcclusionQuality(occlusion.quality).toLowerCase()} quality`,
      })}
      icon={<GradientIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Darken creases and corners"}
      onToggle={onToggle}
    >
      <RenderValueChoice
        label={"Quality"}
        options={RENDER_AMBIENT_OCCLUSION_QUALITY_OPTIONS}
        value={occlusion.quality}
        onChange={(quality: ERendererAmbientOcclusionQuality) => set({ quality })}
      />

      <RenderValueSlider
        label={"Radius"}
        value={occlusion.radius}
        {...RENDER_AMBIENT_OCCLUSION_LIMITS.radius}
        format={formatOcclusionRadius}
        onChange={(radius: number) => set({ radius })}
      />

      <RenderValueSlider
        label={"Strength"}
        value={occlusion.strength}
        {...RENDER_AMBIENT_OCCLUSION_LIMITS.strength}
        format={formatOcclusionStrength}
        onChange={(strength: number) => set({ strength })}
      />

      <Button size={"small"} onClick={reset}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
