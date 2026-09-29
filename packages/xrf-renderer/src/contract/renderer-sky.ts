import { Nullable } from "@xrf/types";

import { IRendererClouds } from "#/contract/renderer-clouds";
import { TRendererColor } from "#/contract/renderer-color";

/**
 * The sky, as the two weather keyframes either side of the time name it (`$user$sky0`, `$user$sky1`).
 */
export interface IRendererSky {
  /** The texture keys of the two keyframes' `sky_texture` cubes. */
  textures: readonly [string, string];
  /** The texture keys of their `#small` irradiance cubes, which light the hemisphere (`env_s0`, `env_s1`). */
  environments: readonly [string, string];
  /** How far from the first to the second, `L_ambient.w`. */
  blend: number;
  /** `sky_color`: what the cubes are multiplied by. */
  color: TRendererColor;
  /** `sky_rotation`, in degrees about the vertical. */
  rotation: number;
  /** The clouds over it, or none. */
  clouds: Nullable<IRendererClouds>;
}
