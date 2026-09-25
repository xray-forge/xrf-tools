import { default as GrassIcon } from "@mui/icons-material/Grass";
import { Button } from "@mui/material";
import { IRendererGrassSettings } from "@xrf/renderer";
import { ReactElement, useCallback } from "react";

import { ILevelFeatureOptions, TLevelGrassOptions } from "@/core/level/lib/features/level-feature-options";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  fromGrassDensityScale,
  RENDER_GRASS_LIMITS,
  toGrassDensityScale,
} from "@/core/render/lib/features/render-feature-choices";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";
import { formatNumber } from "@/lib/format/number";

interface ILevelGrassActionProps extends BaseComponentProps {
  isOn: boolean;
  /** The grass the view is drawn with: the settings', with the view's own values over them. */
  grass: IRendererGrassSettings;
  /** Whether the renderer's settings draw grass at all, which this view can only narrow. */
  isAvailable?: boolean;
  features: ILevelFeatureOptions;
  onToggle: () => void;
  onChange: (features: ILevelFeatureOptions) => void;
}

/**
 * Whether this view plants the level's grass, and how dense, how far and how tall.
 */
export function LevelGrassAction({
  "data-testid": dataTestId = "level-grass-action",
  id,
  className,
  isOn,
  grass,
  isAvailable = true,
  features,
  onToggle,
  onChange,
}: ILevelGrassActionProps): ReactElement {
  const set = useCallback(
    (part: Partial<TLevelGrassOptions>) => onChange({ ...features, grass: { ...features.grass, ...part } }),
    [features, onChange]
  );

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Grass"}
      description={
        !isAvailable
          ? "Grass is off in Settings, under Rendering"
          : isOn
            ? `Grass to ${formatNumber(grass.radius, 0)} m, ${formatNumber(toGrassDensityScale(grass.density), 2)}× the game's density`
            : "Grass off, the ground bare"
      }
      icon={<GrassIcon />}
      isOn={isOn && isAvailable}
      isDisabled={!isAvailable}
      toggleLabel={"Plant the grass"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"Density"}
        value={toGrassDensityScale(grass.density)}
        {...RENDER_GRASS_LIMITS.density}
        format={(value: number) => `${formatNumber(value, 2)}×`}
        onChange={(scale: number) => set({ density: fromGrassDensityScale(scale) })}
      />

      <RenderValueSlider
        label={"Radius"}
        value={grass.radius}
        {...RENDER_GRASS_LIMITS.radius}
        format={(value: number) => `${formatNumber(value, 0)} m`}
        onChange={(radius: number) => set({ radius })}
      />

      <RenderValueSlider
        label={"Height"}
        value={grass.height}
        {...RENDER_GRASS_LIMITS.height}
        format={(value: number) => `${formatNumber(value, 1)}×`}
        onChange={(height: number) => set({ height })}
      />

      <Button size={"small"} onClick={() => onChange({ ...features, grass: {} })}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
