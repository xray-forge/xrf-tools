import { Nullable } from "@xrf/types";

import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IRendererSunPosition } from "#/contract/weather/renderer-sun-position";
import { ERendererWeatherEngine } from "#/contract/weather/renderer-weather-engine";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";

/**
 * A weather cycle the renderer plays by itself: its keyframes, where its sun stands, and where every sky it names
 * is fetched from.
 */
export interface IRendererWeather {
  engine: ERendererWeatherEngine;
  /** Sorted by time, at least one. */
  keyframes: ReadonlyArray<IRendererWeatherKeyframe>;
  /** Twenty-four hours, midnight first, on an engine that stands the sun by its table; null otherwise. */
  sunTable: Nullable<ReadonlyArray<IRendererSunPosition>>;
  /** Where each sky and irradiance cube the keyframes name is fetched from, by reference; one left out is not drawn. */
  textures: Readonly<Record<string, Extract<TRendererTextureSource, { encoding: ERendererTextureEncoding.FETCH }>>>;
}
