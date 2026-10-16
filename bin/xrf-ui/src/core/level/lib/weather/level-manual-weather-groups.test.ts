import { describe, expect, it } from "@jest/globals";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  LEVEL_MANUAL_WEATHER_FOG_KEYS,
  LEVEL_MANUAL_WEATHER_RAIN_KEYS,
  LEVEL_MANUAL_WEATHER_SKY_KEYS,
  LEVEL_MANUAL_WEATHER_SUN_KEYS,
  LEVEL_MANUAL_WEATHER_WIND_KEYS,
} from "@/core/level/lib/weather/level-manual-weather-groups";
import { LEVEL_MANUAL_WEATHER_KEYS } from "@/core/level/lib/weather/level-manual-weather-keys";

/** Keys no group sets back: the water's intensity is edited beside the water's own settings, with no reset of its own. */
const UNGROUPED: ReadonlyArray<keyof ILevelManualWeather> = ["waterIntensity"];

describe("level manual weather groups", () => {
  it("puts every key of the keyframe in exactly one group, so each is set back by one reset", () => {
    const grouped: Array<keyof ILevelManualWeather> = [
      ...LEVEL_MANUAL_WEATHER_SUN_KEYS,
      ...LEVEL_MANUAL_WEATHER_SKY_KEYS,
      ...LEVEL_MANUAL_WEATHER_FOG_KEYS,
      ...LEVEL_MANUAL_WEATHER_RAIN_KEYS,
      ...LEVEL_MANUAL_WEATHER_WIND_KEYS,
    ];

    expect(new Set(grouped).size).toBe(grouped.length);
    expect([...grouped, ...UNGROUPED].sort()).toEqual(
      (Object.keys(LEVEL_MANUAL_WEATHER_KEYS) as Array<keyof ILevelManualWeather>).sort()
    );
  });
});
