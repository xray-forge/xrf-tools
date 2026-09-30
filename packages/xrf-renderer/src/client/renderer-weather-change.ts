import { Nullable } from "@xrf/types";

import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { TRendererWeatherChange } from "#/contract/weather/renderer-weather-change";

/**
 * @param sent - The weather sent before, or null for none.
 * @param weather - The weather to send now.
 * @returns What crosses to the worker: every part not handed over as the same object as before, or the whole weather
 *   after none.
 */
export function toRendererWeatherChange(
  sent: Nullable<IRendererWeather>,
  weather: IRendererWeather
): TRendererWeatherChange {
  if (!sent) {
    return weather;
  }

  const change: Record<string, unknown> = {};

  for (const key of Object.keys(weather) as Array<keyof IRendererWeather>) {
    if (weather[key] !== sent[key]) {
      change[key] = weather[key];
    }
  }

  return change as TRendererWeatherChange;
}
