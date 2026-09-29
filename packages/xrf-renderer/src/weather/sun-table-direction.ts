import { toRadians } from "@xrf/math";
import { Maybe } from "@xrf/types";

import { TRendererVector } from "#/contract/renderer-vector";
import { IRendererSunPosition } from "#/contract/weather/renderer-sun-position";
import { WEATHER_DAY_LENGTH } from "#/weather/weather-day";
import { toWeatherHeading } from "#/weather/weather-heading";

/**
 * `calculate_config_sun_dir`: the table's two hours around a time of day, lerped by the minute.
 *
 * @param positions - The table's twenty-four hours, midnight first.
 * @param time - Seconds since midnight.
 * @returns The way sunlight travels, in engine space.
 */
export function toSunTableDirection(positions: ReadonlyArray<IRendererSunPosition>, time: number): TRendererVector {
  const current: number = time / (WEATHER_DAY_LENGTH / 24);
  const hour: number = Math.floor(current);
  const weight: number = current - hour;
  const from: IRendererSunPosition = at(positions, hour);
  const to: IRendererSunPosition = at(positions, hour + 1);

  return toWeatherHeading(
    toRadians(from.altitude + (to.altitude - from.altitude) * weight),
    toRadians(from.longitude + (to.longitude - from.longitude) * weight)
  );
}

function at(positions: ReadonlyArray<IRendererSunPosition>, hour: number): IRendererSunPosition {
  const position: Maybe<IRendererSunPosition> = positions[hour % 24];

  return position ?? { altitude: 0, longitude: 0 };
}
