import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";

/** Two keyframes, the first blended from and the second towards: `Current[0]` and `Current[1]`. */
export type TWeatherKeyframePair = readonly [IRendererWeatherKeyframe, IRendererWeatherKeyframe];
