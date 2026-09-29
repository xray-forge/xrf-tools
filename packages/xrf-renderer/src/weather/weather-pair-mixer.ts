import { ERendererEngine } from "#/contract/renderer-engine";
import { IRendererWeatherModifier } from "#/contract/weather/renderer-weather-modifier";
import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { TWeatherSun } from "#/weather/weather-sun";

/**
 * Two keyframes and what they are mixed by: `CEnvDescriptorMixer::lerp`, with the modifiers reaching the view.
 */
export interface IWeatherPairMixer {
  pair: TWeatherKeyframePair;
  engine: ERendererEngine;
  sun: TWeatherSun;
  /** The level's `level.env_mod` volumes. */
  modifiers: ReadonlyArray<IRendererWeatherModifier>;
}
