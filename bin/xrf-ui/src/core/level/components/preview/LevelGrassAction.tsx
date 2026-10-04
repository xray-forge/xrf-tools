import { default as GrassIcon } from "@mui/icons-material/Grass";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatGrassDensity,
  formatGrassHeight,
  formatGrassRadius,
  fromGrassDensityScale,
  RENDER_GRASS_LIMITS,
  toGrassDensityScale,
} from "@/core/render/lib/features";
import { TRenderGrassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";

/**
 * Whether this view plants the level's grass, and how dense, how far and how tall.
 */
export function LevelGrassAction({
  "data-testid": dataTestId = "level-grass-action",
  id,
  className,
  isOn,
  state,
  features,
  onToggle,
  onChange,
}: ILevelFeatureActionProps<"grass">): ReactElement {
  const { set, reset } = useLevelFeatureOverride("grass", features, onChange);
  const grass: TRenderGrassSettings = state.value;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Grass"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        label: "Grass",
        off: "Grass off, the ground bare",
        on: `Grass to ${formatGrassRadius(grass.radius)}, ${formatGrassDensity(grass.density)} the game's density`,
      })}
      icon={<GrassIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Plant the grass"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Density"}
        value={toGrassDensityScale(grass.density)}
        {...RENDER_GRASS_LIMITS.density}
        format={(scale: number) => formatGrassDensity(fromGrassDensityScale(scale))}
        onChange={(scale: number) => set({ density: fromGrassDensityScale(scale) })}
      />

      <RenderValueSlider
        label={"Radius"}
        value={grass.radius}
        {...RENDER_GRASS_LIMITS.radius}
        format={formatGrassRadius}
        onChange={(radius: number) => set({ radius })}
      />

      <RenderValueSlider
        label={"Height"}
        value={grass.height}
        {...RENDER_GRASS_LIMITS.height}
        format={formatGrassHeight}
        onChange={(height: number) => set({ height })}
      />

      <Button size={"small"} onClick={reset}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
