import { Nullable } from "@xrf/types";

import { IRendererWeatherEffectReport } from "#/contract/weather/renderer-weather-effect-report";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";

/**
 * Where the renderer's weather stands.
 */
export interface IRendererWeatherReport {
  /** Seconds since midnight. */
  time: number;
  /** The times of the two keyframes blended between, the effect's own while one plays. */
  between: readonly [number, number];
  /** How far from the first to the second. */
  weight: number;
  /** The effect playing, or null for none. */
  effect: Nullable<IRendererWeatherEffectReport>;
  /** How many of the level's modifiers reach the camera. */
  modifiers: number;
  /** What is mixed now as one keyframe, the heavier of the pair's textures: what a hand-set keyframe starts from. */
  current: IRendererWeatherKeyframe;
}
