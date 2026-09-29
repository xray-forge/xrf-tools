import { default as WaterDropIcon } from "@mui/icons-material/WaterDrop";
import { ReactElement } from "react";

import { ILevelManualWeatherActionProps } from "@/core/level/components/weather/level-manual-weather-action-props";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { toLevelWeatherTriple } from "@/core/level/lib/weather/level-weather-vector";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { formatPercent } from "@/lib/format/number";

/** The keys the popover sets. */
const RAIN_KEYS: ReadonlyArray<keyof ILevelManualWeather> = ["rainDensity", "rainColor"];

/**
 * How hard it rains and the streaks' colour; whether it rains at all.
 */
export function LevelRainAction({
  "data-testid": dataTestId = "level-rain-action",
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
      label={"Rain"}
      description={
        !isOn ? "Rain off" : manual.rainDensity > 0 ? `Rain at ${formatPercent(manual.rainDensity)}` : "No rain"
      }
      icon={<WaterDropIcon />}
      isOn={isOn}
      toggleLabel={"Rain where the weather rains"}
      onToggle={onToggle}
    >
      <RenderValueSlider
        label={"rain_density"}
        value={manual.rainDensity}
        {...LEVEL_MANUAL_WEATHER_LIMITS.rainDensity}
        format={formatPercent}
        onChange={(rainDensity: number) => onEdit({ rainDensity })}
      />

      <LevelWeatherVectorField
        label={"rain_color"}
        isColor
        value={manual.rainColor}
        onChange={(rainColor) => onEdit({ rainColor: toLevelWeatherTriple(rainColor) })}
      />

      <LevelWeatherResetButton keys={RAIN_KEYS} onEdit={onEdit} />
    </EditorPopoverToggle>
  );
}
