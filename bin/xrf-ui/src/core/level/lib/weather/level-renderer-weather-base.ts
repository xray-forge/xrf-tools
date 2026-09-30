import { IRendererWeather } from "@xrf/renderer";

/**
 * What every weather of one level plays with, whatever keyframes it plays: built once for the level, so each weather
 * built over it hands the renderer the same parts.
 */
export type TLevelRendererWeatherBase = Pick<
  IRendererWeather,
  "engine" | "effects" | "modifiers" | "rain" | "thunder" | "wet" | "sunTable" | "textures"
>;
