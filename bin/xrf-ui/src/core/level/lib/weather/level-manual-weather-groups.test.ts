import { describe, expect, it } from "@jest/globals";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";
import {
  LEVEL_MANUAL_WEATHER_FOG_KEYS,
  LEVEL_MANUAL_WEATHER_RAIN_KEYS,
  LEVEL_MANUAL_WEATHER_SKY_KEYS,
  LEVEL_MANUAL_WEATHER_SUN_KEYS,
  LEVEL_MANUAL_WEATHER_WATER_KEYS,
  LEVEL_MANUAL_WEATHER_WIND_KEYS,
} from "@/core/level/lib/weather/level-manual-weather-groups";
import { LEVEL_MANUAL_WEATHER_KEYS } from "@/core/level/lib/weather/level-manual-weather-keys";

describe("level manual weather groups", () => {
  it("puts every key of the keyframe in exactly one group, so each is set back by one reset", () => {
    const grouped: Array<keyof ILevelManualWeather> = [
      ...LEVEL_MANUAL_WEATHER_SUN_KEYS,
      ...LEVEL_MANUAL_WEATHER_SKY_KEYS,
      ...LEVEL_MANUAL_WEATHER_FOG_KEYS,
      ...LEVEL_MANUAL_WEATHER_RAIN_KEYS,
      ...LEVEL_MANUAL_WEATHER_WIND_KEYS,
      ...LEVEL_MANUAL_WEATHER_WATER_KEYS,
    ];

    expect(new Set(grouped).size).toBe(grouped.length);
    expect(grouped.sort()).toEqual((Object.keys(LEVEL_MANUAL_WEATHER_KEYS) as Array<keyof ILevelManualWeather>).sort());
  });
});
