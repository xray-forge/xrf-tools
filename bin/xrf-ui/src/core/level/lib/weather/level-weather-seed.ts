import { Nullable } from "@xrf/types";

import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";

/**
 * What a keyframe set by hand was seeded from: the cycle playing, the time its clock stood at, and what it showed.
 */
export interface ILevelWeatherSeed {
  cycle: string;
  /** Seconds since midnight. */
  time: number;
  /** The keyframe the weather showed then, which a reset sets keys back to; none where an older memory kept it not. */
  weather: Nullable<ILevelManualWeather>;
}
