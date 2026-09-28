/**
 * The grass (`CDetailManager`): planted on the GPU around the camera as the engine plants it, and drawn into the
 * G-buffer. The engine's are 49 metres round at a density of 0.6 (`r__detail_radius`, `r__detail_density`).
 */
export interface IRendererGrassSettings {
  isEnabled: boolean;
  /** How far apart a slot's candidates stand, from 0.1 (the densest) to 0.99 (the sparsest): `r__detail_density`. */
  density: number;
  /** Whole metres around the camera grass is planted to, from 49: `r__detail_radius`. */
  radius: number;
  /** What every planted tuft is scaled by, 1 as the engine plants it: `r__detail_height`. */
  height: number;
}

/** The engine's own grass. */
export const DEFAULT_RENDERER_GRASS_SETTINGS: IRendererGrassSettings = {
  density: 0.6,
  height: 1,
  isEnabled: true,
  radius: 49,
};
