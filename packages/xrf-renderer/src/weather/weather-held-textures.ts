import { Maybe } from "@xrf/types";

import { IRendererWeather } from "#/contract/weather/renderer-weather";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IWeatherHeldTexturesInput } from "#/weather/weather-held-textures-input";
import { WeatherPair } from "#/weather/weather-pair";

/**
 * The textures a frame of weather holds, by reference: the pair's skies and the next keyframe's, fetched before the
 * clock reaches it, and what the rain and the bolts draw with, held while the weather plays so a shower or a strike
 * starting has them.
 *
 * @param input - The weather, the keyframes playing and the pair mixed.
 * @returns The references, a reference as often as it is named.
 */
export function listWeatherTextures(input: IWeatherHeldTexturesInput): Array<string> {
  const { weather, keyframes, pair } = input;
  const next: Maybe<IRendererWeatherKeyframe> = WeatherPair.selectNext(keyframes, pair[1].time + 0.5);

  return [
    ...[pair[0], pair[1], next].flatMap((keyframe: Maybe<IRendererWeatherKeyframe>) =>
      keyframe ? [keyframe.skyTexture, keyframe.skyTextureEnv, keyframe.cloudsTexture] : []
    ),
    ...listRain(weather),
    ...listThunder(weather),
  ];
}

/** The rain's textures and the flow it wets walls with; the splashes' volume is the wet surfaces' own to fetch. */
function listRain(weather: IRendererWeather): Array<string> {
  const { rain, wet } = weather;

  return [...(rain ? [rain.streak, ...(rain.drop ? [rain.drop.texture] : [])] : []), ...(wet ? [wet.flow] : [])];
}

/** The bolts' models' textures and their glows'. */
function listThunder(weather: IRendererWeather): Array<string> {
  const { thunder } = weather;

  return thunder
    ? [
        ...thunder.models.map((model) => model.texture),
        ...Object.values(thunder.bolts).flatMap((bolt) => [bolt.top.texture, bolt.center.texture]),
      ]
    : [];
}
