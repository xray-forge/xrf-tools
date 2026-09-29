import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";

/**
 * What an effect is started from.
 */
export interface IWeatherEffectStart {
  name: string;
  /** The effect's keyframes, sorted by their time from its start. */
  effect: ReadonlyArray<IRendererWeatherKeyframe>;
  /** The cycle it plays over, sorted by time. */
  cycle: ReadonlyArray<IRendererWeatherKeyframe>;
  /** Seconds since midnight it starts at. */
  time: number;
  /** Game seconds a real second, which the lead-in is timed by. */
  factor: number;
}
