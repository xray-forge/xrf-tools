import { default as GrassIcon } from "@mui/icons-material/Grass";
import { Button } from "@mui/material";
import { ReactElement, useCallback } from "react";

import { ERenderFoliageMode, RenderFoliageMode } from "@/core/ipc/types/xrf-renderer";
import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { RenderValueChoice } from "@/core/render/components/controls/RenderValueChoice";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import {
  formatFoliageStrength,
  formatGrassDensity,
  formatGrassHeight,
  formatGrassRadius,
  fromGrassDensityScale,
  IRenderChoiceOption,
  RENDER_FOLIAGE_LIMITS,
  RENDER_GRASS_LIMITS,
  toGrassDensityScale,
} from "@/core/render/lib/features";
import { TRenderFoliageSettings, TRenderGrassSettings } from "@/core/render/lib/settings/render-feature-settings";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { formatPercent } from "@/lib/format/number";

const FOLIAGE_MODE_OPTIONS: ReadonlyArray<IRenderChoiceOption<RenderFoliageMode>> = [
  { label: "Engine", value: ERenderFoliageMode.ENGINE },
  { label: "Enhanced", value: ERenderFoliageMode.ENHANCED },
];

/** The enhanced foliage motion's strengths, each a slider. */
const FOLIAGE_STRENGTHS: ReadonlyArray<[keyof typeof RENDER_FOLIAGE_LIMITS, string]> = [
  ["grassSpeed", "Grass speed"],
  ["grassTurbulence", "Grass turbulence"],
  ["grassPush", "Grass push"],
  ["grassWave", "Grass wave"],
  ["treesSpeed", "Branches speed"],
  ["treesTrunk", "Trunk speed"],
  ["treesBend", "Trunk bend"],
  ["sssIntensity", "Light through leaves"],
  ["sssColor", "Sun colour through leaves"],
];

/**
 * Whether this view plants the level's grass, how dense, how far and how tall, and how it and the trees move in the
 * wind.
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
  const foliage: TRenderFoliageSettings = grass.foliage;

  const setFoliage = useCallback(
    (part: Partial<TRenderFoliageSettings>): void => {
      set({ foliage: { ...foliage, ...part } });
    },
    [foliage, set]
  );

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

      <RenderValueChoice
        label={"Foliage motion"}
        options={FOLIAGE_MODE_OPTIONS}
        value={foliage.mode}
        onChange={(mode: RenderFoliageMode) => setFoliage({ mode })}
      />

      {foliage.mode === ERenderFoliageMode.ENHANCED ? (
        <>
          <RenderValueSlider
            label={"Least wind"}
            value={foliage.minSpeed}
            {...RENDER_FOLIAGE_LIMITS.minSpeed}
            format={formatPercent}
            onChange={(minSpeed: number) => setFoliage({ minSpeed })}
          />

          {FOLIAGE_STRENGTHS.map(([key, label]) => (
            <RenderValueSlider
              key={key}
              label={label}
              value={foliage[key]}
              {...RENDER_FOLIAGE_LIMITS[key]}
              format={formatFoliageStrength}
              onChange={(value: number) => setFoliage({ [key]: value })}
            />
          ))}
        </>
      ) : null}

      <Button size={"small"} onClick={reset}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
