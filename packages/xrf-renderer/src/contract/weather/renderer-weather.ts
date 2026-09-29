import { Nullable } from "@xrf/types";

import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IRendererRain } from "#/contract/weather/renderer-rain";
import { IRendererSunPosition } from "#/contract/weather/renderer-sun-position";
import { IRendererThunder } from "#/contract/weather/renderer-thunder";
import { ERendererWeatherEngine } from "#/contract/weather/renderer-weather-engine";
import { IRendererWeatherKeyframe } from "#/contract/weather/renderer-weather-keyframe";
import { IRendererWeatherModifier } from "#/contract/weather/renderer-weather-modifier";
import { IRendererWetSurfaces } from "#/contract/weather/renderer-wet-surfaces";

/**
 * A weather cycle the renderer plays by itself: its keyframes, where its sun stands, the effects it can play over it,
 * the level's modifiers, and where every texture they name is fetched from.
 */
export interface IRendererWeather {
  engine: ERendererWeatherEngine;
  /** Sorted by time, at least one. */
  keyframes: ReadonlyArray<IRendererWeatherKeyframe>;
  /** Twenty-four hours, midnight first, on an engine that stands the sun by its table; null otherwise. */
  sunTable: Nullable<ReadonlyArray<IRendererSunPosition>>;
  /** Every weather effect by name, its keyframes sorted by their time from its start. */
  effects: Readonly<Record<string, ReadonlyArray<IRendererWeatherKeyframe>>>;
  /** The level's `level.env_mod` volumes. */
  modifiers: ReadonlyArray<IRendererWeatherModifier>;
  /** What its rain is drawn with, or null for a weather that draws none. */
  rain: Nullable<IRendererRain>;
  /** What it strikes with, or null for a weather that strikes nothing. */
  thunder: Nullable<IRendererThunder>;
  /** What its rain wets surfaces with, or null for none. */
  wet: Nullable<IRendererWetSurfaces>;
  /** Where each texture the keyframes name is fetched from, by reference; one left out is not drawn. */
  textures: Readonly<Record<string, Extract<TRendererTextureSource, { encoding: ERendererTextureEncoding.FETCH }>>>;
}
