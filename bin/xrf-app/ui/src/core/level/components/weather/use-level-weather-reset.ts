import { useInjection } from "@wirestate/react";
import { Optional } from "@xrf/types";
import { useCallback } from "react";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LevelWeatherService } from "@/core/level/services/level-weather";

/** What a reset of keys set by hand says it does: they stand as the weather they were seeded from again. */
export const LEVEL_WEATHER_RESET_LABEL: string = "Reset to the weather";

/**
 * @param keys - Keys of the keyframe set by hand, as a popover section heads them; a group from
 *   `level-manual-weather-groups`, so the reference is stable between renders.
 * @returns The section heading's reset to the seeded weather, or none while the weather lights the level or no key
 *   differs from its seed.
 */
export function useLevelWeatherReset(keys: ReadonlyArray<keyof ILevelManualWeather>): Optional<() => void> {
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);

  const reset = useCallback(() => weatherService.resetManual(keys), [weatherService, keys]);

  return weatherService.isManualResettable(keys) ? reset : undefined;
}
