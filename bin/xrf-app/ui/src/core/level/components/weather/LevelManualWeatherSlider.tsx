import { BaseComponentProps } from "@xrf/ui/element-types";
import { ReactElement } from "react";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  describeLevelManualWeatherKey,
  LEVEL_MANUAL_WEATHER_LABELS,
  LEVEL_MANUAL_WEATHER_NEGATED,
} from "@/core/level/lib/weather/level-manual-weather-labels";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { TLevelManualWeatherNumberKey } from "@/core/level/lib/weather/level-manual-weather-number-key";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";

interface ILevelManualWeatherSliderProps extends BaseComponentProps {
  /** The number it sets, offered between its limits, its config key in the tooltip. */
  field: TLevelManualWeatherNumberKey;
  /** The keyframe on screen. */
  manual: ILevelManualWeather;
  format: (value: number) => string;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * One number of the keyframe set by hand, on a slider.
 */
export function LevelManualWeatherSlider({
  "data-testid": dataTestId = "level-manual-weather-slider",
  id,
  className,
  field,
  manual,
  format,
  onEdit,
}: ILevelManualWeatherSliderProps): ReactElement {
  const sign: number = LEVEL_MANUAL_WEATHER_NEGATED.has(field) ? -1 : 1;

  return (
    <RenderValueSlider
      data-testid={dataTestId}
      id={id}
      className={className}
      label={LEVEL_MANUAL_WEATHER_LABELS[field]}
      hint={describeLevelManualWeatherKey(field)}
      value={sign * manual[field]}
      {...LEVEL_MANUAL_WEATHER_LIMITS[field]}
      format={format}
      onChange={(value: number) => onEdit({ [field]: sign * value })}
    />
  );
}
