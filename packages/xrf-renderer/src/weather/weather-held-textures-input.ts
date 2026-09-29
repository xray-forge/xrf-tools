import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";

/**
 * What a frame of weather holds textures for: the weather, the keyframes playing, cycle or effect, and the pair mixed.
 */
export interface IWeatherHeldTexturesInput {
  weather: IRendererWeather;
  keyframes: ReadonlyArray<IRendererWeatherKeyframe>;
  pair: TWeatherKeyframePair;
}
