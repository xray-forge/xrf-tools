import { DEFAULT_LEVEL_MANUAL_WEATHER, ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { TLevelManualWeatherVectorKey } from "@/core/level/lib/weather/level-manual-weather-vector-key";

/**
 * @param key - The key a vector field edits.
 * @param values - What it was edited to.
 * @returns The edit, as many components as the key holds, zero where the field had fewer.
 */
export function toLevelManualWeatherVectorPatch(
  key: TLevelManualWeatherVectorKey,
  values: ReadonlyArray<number>
): Partial<ILevelManualWeather> {
  const held: ReadonlyArray<number> = DEFAULT_LEVEL_MANUAL_WEATHER[key];

  return { [key]: held.map((_: number, index: number) => values[index] ?? 0) };
}
