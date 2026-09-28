import { TRendererColor } from "#/contract/renderer-color";

/**
 * The sky, as the two weather keyframes either side of the time name it (`$user$sky0`, `$user$sky1`).
 */
export interface IRendererSky {
  /** The texture keys of the two keyframes' `sky_texture` cubes. */
  textures: readonly [string, string];
  /** How far from the first to the second, `L_ambient.w`. */
  blend: number;
  /** `sky_color`: what the cubes are multiplied by. */
  color: TRendererColor;
  /** `sky_rotation`, in degrees about the vertical. */
  rotation: number;
}
