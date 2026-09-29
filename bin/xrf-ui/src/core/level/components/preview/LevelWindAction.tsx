import { default as AirIcon } from "@mui/icons-material/Air";
import { ReactElement } from "react";

import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
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
      <LevelManualWeatherSlider
        field={"windVelocity"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 1)} m/s`}
        onEdit={onEdit}
      />

      <LevelManualWeatherSlider field={"windDirection"} manual={manual} format={formatDegrees} onEdit={onEdit} />

      <LevelManualWeatherSlider
        field={"treesAmplitude"}
        manual={manual}
        format={(value: number) => formatNumber(value, 3)}
        onEdit={onEdit}
      />

      <LevelManualWeatherSlider
        field={"treesSpeed"}
        manual={manual}
        format={(value: number) => formatNumber(value, 2)}
        onEdit={onEdit}
      />

      <LevelManualWeatherSlider field={"treesRotation"} manual={manual} format={formatDegrees} onEdit={onEdit} />

      <LevelManualWeatherVectorField field={"treesWave"} manual={manual} onEdit={onEdit} />

      <LevelWeatherResetButton keys={WIND_KEYS} onEdit={onEdit} />
    </EditorPopoverToggle>
  );
}
