import { ReactElement } from "react";

import { LevelWeatherVectorField } from "@/core/level/components/weather/LevelWeatherVectorField";
import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  describeLevelManualWeatherKey,
  LEVEL_MANUAL_WEATHER_LABELS,
} from "@/core/level/lib/weather/level-manual-weather-labels";
import { TLevelManualWeatherVectorKey } from "@/core/level/lib/weather/level-manual-weather-vector-key";
import { toLevelManualWeatherVectorPatch } from "@/core/level/lib/weather/level-weather-vector";
import { BaseComponentProps } from "@/lib/dom/element-types";

interface ILevelManualWeatherVectorFieldProps extends BaseComponentProps {
  /** The numbers it sets, its config key in the tooltip. */
  field: TLevelManualWeatherVectorKey;
  /** The keyframe on screen. */
  manual: ILevelManualWeather;
  /** Whether it is a colour. */
  isColor?: boolean;
  onEdit: (patch: Partial<ILevelManualWeather>) => void;
}

/**
 * A key of the keyframe set by hand holding a few numbers, each typed as the engine holds it.
 */
export function LevelManualWeatherVectorField({
  "data-testid": dataTestId = "level-manual-weather-vector-field",
  id,
  className,
  field,
  manual,
  isColor,
  onEdit,
}: ILevelManualWeatherVectorFieldProps): ReactElement {
  return (
    <LevelWeatherVectorField
      data-testid={dataTestId}
      id={id}
      className={className}
      label={LEVEL_MANUAL_WEATHER_LABELS[field]}
      hint={describeLevelManualWeatherKey(field)}
      value={manual[field]}
      isColor={isColor}
      onChange={(values: Array<number>) => onEdit(toLevelManualWeatherVectorPatch(field, values))}
    />
  );
}
