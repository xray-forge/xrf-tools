import { ILevelManualWeather } from "@/core/level/lib/weather/level-manual-weather";

/** A key of a keyframe set by hand holding a few numbers: a colour, or the trees' wave. */
export type TLevelManualWeatherVectorKey = {
  [K in keyof ILevelManualWeather]: ILevelManualWeather[K] extends ReadonlyArray<number> ? K : never;
}[keyof ILevelManualWeather];
