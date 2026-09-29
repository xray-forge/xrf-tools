import { ReactElement } from "react";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LEVEL_MANUAL_WEATHER_KEYS } from "@/core/level/lib/weather/level-manual-weather-keys";
import { LEVEL_MANUAL_WEATHER_LIMITS } from "@/core/level/lib/weather/level-manual-weather-limits";
import { TLevelManualWeatherNumberKey } from "@/core/level/lib/weather/level-manual-weather-number-key";
import { RenderValueSlider } from "@/core/render/components/controls/RenderValueSlider";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelManualWeatherSliderProps extends BaseComponentProps {
  /** The number it sets, named by the key a weather config writes it under and offered between its limits. */
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
  return (
    <RenderValueSlider
      data-testid={dataTestId}
      id={id}
      className={className}
      label={LEVEL_MANUAL_WEATHER_KEYS[field]}
      value={manual[field]}
      {...LEVEL_MANUAL_WEATHER_LIMITS[field]}
      format={format}
      onChange={(value: number) => onEdit({ [field]: value })}
    />
  );
}
