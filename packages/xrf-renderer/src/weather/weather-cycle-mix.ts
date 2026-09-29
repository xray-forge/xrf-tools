import { IWeatherMix } from "#/weather/weather-mix";

/**
 * A cycle mixed at one time of day, and which of its keyframes the mix is of.
 */
export interface IWeatherCycleMix extends IWeatherMix {
  /** The keyframes either side, by index. */
  keyframes: readonly [number, number];
}
