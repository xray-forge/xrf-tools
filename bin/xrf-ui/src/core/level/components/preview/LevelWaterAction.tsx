import { default as WaterIcon } from "@mui/icons-material/Water";
import { Button } from "@mui/material";
import { ReactElement } from "react";

import { ILevelFeatureActionProps } from "@/core/level/components/preview/level-feature-action-props";
import { useLevelFeatureOverride } from "@/core/level/components/preview/use-level-feature-override";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { describeLevelFeatureToggle } from "@/core/level/lib/features";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { IRendererWaterSettings } from "@/core/render/lib/contract/renderer-water-settings";
import {
  formatWaterDistortion,
  formatWaterMultiple,
  formatWaveHeight,
  formatWaveSpeed,
  RENDER_WATER_LIMITS,
} from "@/core/render/lib/features";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { CheckboxFormRow } from "@/core/ui/form/CheckboxFormRow";
import { formatPercent } from "@/lib/format/number";

interface ILevelWaterActionProps extends ILevelFeatureActionProps<"water"> {
  /** The keyframe on screen, whose `water_intensity` scales what the water reflects. */
  manual: ILevelManualWeather;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * Whether this view draws the water, how strongly the weather has it reflect, and how it moves, reflects and
 * distorts: the engine's own by default.
 */
export function LevelWaterAction({
  "data-testid": dataTestId = "level-water-action",
  id,
  className,
  isOn,
  state,
  features,
  manual,
  onToggle,
  onChange,
  onEdit,
}: ILevelWaterActionProps): ReactElement {
  const { set, reset } = useLevelFeatureOverride("water", features, onChange);
  const water: IRendererWaterSettings = state.value;

  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Water"}
      description={describeLevelFeatureToggle({
        isAvailable: state.isAvailable,
        isOn,
        label: "Water",
        off: "Water off, what lies under it showing",
        on: `Water reflecting the sky${water.isSoft ? ", soft" : ""}${water.isDistorted ? ", distorting" : ""}`,
      })}
      icon={<WaterIcon />}
      isOn={isOn && state.isAvailable}
      isDisabled={!state.isAvailable}
      toggleLabel={"Draw the water"}
      onToggle={onToggle}
    >
      <LevelManualWeatherSlider field={"waterIntensity"} manual={manual} format={formatPercent} onEdit={onEdit} />

      <CheckboxFormRow
        label={"Soft"}
        description={"Fades by how deep it is, and lays foam in the shallows: r2_soft_water."}
        isChecked={water.isSoft}
        onChange={(isSoft: boolean) => set({ isSoft })}
      />

      <CheckboxFormRow
        label={"Distortion"}
        description={"Moves what is seen through it."}
        isChecked={water.isDistorted}
        onChange={(isDistorted: boolean) => set({ isDistorted })}
      />

      <RenderValueSlider
        label={"Wave height"}
        value={water.waveHeight}
        {...RENDER_WATER_LIMITS.waveHeight}
        format={formatWaveHeight}
        onChange={(waveHeight: number) => set({ waveHeight })}
      />

      <RenderValueSlider
        label={"Wave speed"}
        value={water.waveSpeed}
        {...RENDER_WATER_LIMITS.waveSpeed}
        format={formatWaveSpeed}
        onChange={(waveSpeed: number) => set({ waveSpeed })}
      />

      <RenderValueSlider
        label={"Ripple"}
        value={water.ripple}
        {...RENDER_WATER_LIMITS.ripple}
        format={formatWaterMultiple}
        onChange={(ripple: number) => set({ ripple })}
      />

      <RenderValueSlider
        label={"Reflection"}
        value={water.reflection}
        {...RENDER_WATER_LIMITS.reflection}
        format={formatWaterMultiple}
        onChange={(reflection: number) => set({ reflection })}
      />

      <RenderValueSlider
        label={"Distortion strength"}
        value={water.distortion}
        {...RENDER_WATER_LIMITS.distortion}
        format={formatWaterDistortion}
        onChange={(distortion: number) => set({ distortion })}
      />

      <Button size={"small"} onClick={reset}>
        Back to the settings
      </Button>
    </EditorPopoverToggle>
  );
}
