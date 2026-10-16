import { default as AirIcon } from "@mui/icons-material/Air";
import { useInjection } from "@wirestate/react";
import { formatDegrees, formatNumber } from "@xrf/format";
import { ReactElement } from "react";

import { useLevelViewToggle } from "@/core/level/components/preview/toolbar/use-level-view-toggle";
import { LevelManualWeatherSlider } from "@/core/level/components/weather/LevelManualWeatherSlider";
import { LevelManualWeatherVectorField } from "@/core/level/components/weather/LevelManualWeatherVectorField";
import { LevelWeatherResetButton } from "@/core/level/components/weather/LevelWeatherResetButton";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_WIND_KEYS } from "@/core/level/lib/weather/level-manual-weather-groups";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { EditorPopoverToggle } from "@/core/shell/editor/EditorPopoverToggle";
import { BaseComponentProps } from "@/lib/dom/element-types";

/**
 * The wind the rain drifts on and the trees' sway; whether the trees and the grass sway at all.
 */
export function LevelWindAction({
  "data-testid": dataTestId = "level-wind-action",
  id,
  className,
}: BaseComponentProps): ReactElement {
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);
  const { isOn, toggle } = useLevelViewToggle("isWindy");

  const manual: ILevelManualWeather = weatherService.shown;

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
      onToggle={toggle}
    >
      <LevelManualWeatherSlider
        field={"windVelocity"}
        manual={manual}
        format={(value: number) => `${formatNumber(value, 1)} m/s`}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"windDirection"}
        manual={manual}
        format={formatDegrees}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"treesAmplitude"}
        manual={manual}
        format={(value: number) => formatNumber(value, 3)}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"treesSpeed"}
        manual={manual}
        format={(value: number) => formatNumber(value, 2)}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherSlider
        field={"treesRotation"}
        manual={manual}
        format={formatDegrees}
        onEdit={weatherService.editManual}
      />

      <LevelManualWeatherVectorField field={"treesWave"} manual={manual} onEdit={weatherService.editManual} />

      <LevelWeatherResetButton keys={LEVEL_MANUAL_WEATHER_WIND_KEYS} onReset={weatherService.resetManual} />
    </EditorPopoverToggle>
  );
}
