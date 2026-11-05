import { useInjection } from "@wirestate/react";
import { Optional } from "@xrf/types";
import { useCallback } from "react";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import { LevelWeatherService } from "@/core/level/services/level-weather";

/** What a reset of keys set by hand says it does: they stand as the weather they were seeded from again. */
export const LEVEL_WEATHER_RESET_LABEL: string = "Reset to the weather";

/**
 * @param keys - Keys of the keyframe set by hand, as a popover section heads them; one group of
 *   `level-manual-weather-groups`, so the reference stays the same between renders.
 * @returns What sets them back to the weather the keyframe was seeded from, for the section's heading, while that
 *   changes one of them; none while the weather lights the level or each stands as it was seeded.
 */
export function useLevelWeatherReset(keys: ReadonlyArray<keyof ILevelManualWeather>): Optional<() => void> {
  const weatherService: LevelWeatherService = useInjection(LevelWeatherService);

  const reset = useCallback(() => weatherService.resetManual(keys), [weatherService, keys]);

  return weatherService.isManualResettable(keys) ? reset : undefined;
}
