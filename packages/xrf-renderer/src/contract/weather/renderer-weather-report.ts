import { Nullable } from "@xrf/types";

import { IRendererWeatherEffectReport } from "#/contract/weather/renderer-weather-effect-report";

/**
 * Where the renderer's weather stands.
 */
export interface IRendererWeatherReport {
  /** Seconds since midnight. */
  time: number;
  /** The cycle's keyframes either side, by index, or null while an effect plays. */
  keyframes: Nullable<readonly [number, number]>;
  /** How far from the first to the second. */
  weight: number;
  /** The effect playing, or null for none. */
  effect: Nullable<IRendererWeatherEffectReport>;
  /** How many of the level's modifiers reach the camera. */
  modifiers: number;
}
