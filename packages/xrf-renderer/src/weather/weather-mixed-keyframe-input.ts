import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { IWeatherMix } from "#/weather/weather-mix";

/**
 * What a mix is taken back into one keyframe from: the pair mixed, their mix, and the time it stands at.
 */
export interface IWeatherMixedKeyframeInput {
  pair: TWeatherKeyframePair;
  mix: IWeatherMix;
  /** Seconds since midnight. */
  time: number;
}
