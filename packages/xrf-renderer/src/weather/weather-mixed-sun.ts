import { TRendererVector } from "#/contract/renderer-vector";

/** The sun a mix stands. */
export interface IWeatherMixedSun {
  color: TRendererVector;
  /** The direction its light travels, normalised, in engine space. */
  direction: TRendererVector;
}
