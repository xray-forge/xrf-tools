import { Nullable } from "@xrf/types";

/**
 * The clouds, as the two weather keyframes either side of the time name them (`s_clouds0`, `s_clouds1`).
 */
export interface IRendererClouds {
  /** The texture keys of the two keyframes' `clouds_texture`, or null for one that names none, which draws nothing. */
  textures: readonly [Nullable<string>, Nullable<string>];
  /** `clouds_color` as the engine holds it: the colour scaled by its multiplier, then the cover in alpha. */
  color: readonly [number, number, number, number];
  /** `clouds_rotation`, in degrees about the vertical. */
  rotation: number;
}
