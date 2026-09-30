import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { TWeatherSun } from "#/weather/weather-sun";

/** What a mix's sun is mixed from. */
export interface IWeatherSunMixInput {
  /** Where the sun is stood from. */
  sun: TWeatherSun;
  pair: TWeatherKeyframePair;
  /** Seconds since midnight. */
  time: number;
  /** How far from the first keyframe to the second. */
  weight: number;
}
