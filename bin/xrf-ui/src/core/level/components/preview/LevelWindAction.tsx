import { default as AirIcon } from "@mui/icons-material/Air";
import { ReactElement } from "react";

import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { toLevelWeatherTriple } from "@/core/level/lib/weather/level-weather-vector";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { formatDegrees } from "@/lib/format/angle";
import { formatNumber } from "@/lib/format/number";

/** The keys the popover sets. */
const WIND_KEYS: ReadonlyArray<keyof ILevelManualWeather> = [
  "windVelocity",
  "windDirection",
  "treesAmplitude",
  "treesSpeed",
  "treesRotation",
  "treesWave",
];

/**
 * The wind the rain drifts on and the trees' sway; whether the trees and the grass sway at all.
 */
export function LevelWindAction({
  "data-testid": dataTestId = "level-wind-action",
  id,
  className,
  isOn,
  manual,
  onToggle,
  onEdit,
}: ILevelManualWeatherActionProps): ReactElement {
  return (
    <EditorPopoverToggle
      data-testid={dataTestId}
      id={id}
      className={className}
      label={"Wind"}
      description={
        isOn
          ? `Wind ${formatNumber(manual.windVelocity, 1)} m/s, trees sway ${formatNumber(manual.treesAmplitude, 3)}`
          : "Wind off, trees stand still"
      }
      icon={<AirIcon />}
      isOn={isOn}
      toggleLabel={"Sway the trees"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"wind_velocity"}
        value={manual.windVelocity}
        {...LEVEL_MANUAL_WEATHER_LIMITS.windVelocity}
        format={(value: number) => `${formatNumber(value, 1)} m/s`}
        onChange={(windVelocity: number) => onEdit({ windVelocity })}
      />

      <RenderValueSlider
        label={"wind_direction"}
        value={manual.windDirection}
        {...LEVEL_MANUAL_WEATHER_LIMITS.windDirection}
        format={formatDegrees}
        onChange={(windDirection: number) => onEdit({ windDirection })}
      />

      <RenderValueSlider
        label={"trees_amplitude"}
        value={manual.treesAmplitude}
        {...LEVEL_MANUAL_WEATHER_LIMITS.treesAmplitude}
        format={(value: number) => formatNumber(value, 3)}
        onChange={(treesAmplitude: number) => onEdit({ treesAmplitude })}
      />

      <RenderValueSlider
        label={"trees_speed"}
        value={manual.treesSpeed}
        {...LEVEL_MANUAL_WEATHER_LIMITS.treesSpeed}
        format={(value: number) => formatNumber(value, 2)}
        onChange={(treesSpeed: number) => onEdit({ treesSpeed })}
      />

      <RenderValueSlider
        label={"trees_rotation"}
        value={manual.treesRotation}
        {...LEVEL_MANUAL_WEATHER_LIMITS.treesRotation}
        format={formatDegrees}
        onChange={(treesRotation: number) => onEdit({ treesRotation })}
      />

      <LevelWeatherVectorField
        label={"trees_wave"}
        value={manual.treesWave}
        onChange={(treesWave) => onEdit({ treesWave: toLevelWeatherTriple(treesWave) })}
      />

      <LevelWeatherResetButton keys={WIND_KEYS} onEdit={onEdit} />
    </EditorPopoverToggle>
  );
}
