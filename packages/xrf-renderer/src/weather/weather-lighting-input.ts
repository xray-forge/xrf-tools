import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherControl } from "#/contract/weather/renderer-weather-control";
import { TWeatherKeyframePair } from "#/weather/weather-keyframe-pair";
import { IWeatherMix } from "#/weather/weather-mix";

/**
 * What a frame of weather is lit from: the weather playing, the pair of keyframes mixed and their mix, and the view's
 * switches.
 */
export interface IWeatherLightingInput {
  weather: IRendererWeather;
  pair: TWeatherKeyframePair;
  mix: IWeatherMix;
  control: IRendererWeatherControl;
}
