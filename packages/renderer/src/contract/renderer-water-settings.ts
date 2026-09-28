/**
 * The water (`water.vs`, `water.ps`, `waterd.ps`): rippled and reflecting the sky, blended over the depth behind it
 * and distorting it. The engine's own look by default: its constants are `shared/waterconfig.h`'s and `def_distort`.
 */
export interface IRendererWaterSettings {
  /** Whether the water is drawn at all; off, what lies under it shows. */
  isEnabled: boolean;
  /** `r2_soft_water`: soft water fades by the depth behind it, darkens with it and lays foam in the shallows. */
  isSoft: boolean;
  /** Whether water moves what is seen through it, as the engine's distortion target does. */
  isDistorted: boolean;
  /** How high the waves lift the surface, in metres: `W_POSITION_SHIFT_HEIGHT`. */
  waveHeight: number;
  /** How fast they run: `W_POSITION_SHIFT_SPEED`. */
  waveSpeed: number;
  /** What the two normal layers' scroll is multiplied by, one as `W_DISTORT_AMP_0` and `_1` scroll them. */
  ripple: number;
  /** What the sky's reflection is multiplied by, one as the engine mixes it. */
  reflection: number;
  /** How far the distortion moves what is behind it, a share of the screen: `def_distort`. */
  distortion: number;
}

/** The engine's own water. */
export const DEFAULT_RENDERER_WATER_SETTINGS: IRendererWaterSettings = {
  distortion: 0.05,
  isDistorted: true,
  isEnabled: true,
  isSoft: true,
  reflection: 1,
  ripple: 1,
  waveHeight: 1 / 60,
  waveSpeed: 25,
};
