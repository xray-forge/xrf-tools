/**
 * Everything about how a level is framed, as one value.
 */
export interface ILevelRenderConfig {
  /** What the canvas shows where nothing is drawn and no fog is. */
  backgroundColor: number;
  /** Where the camera starts seeing, `VIEWPORT_NEAR` (`xrEngine/device.h`). */
  cameraNear: number;
  cameraFar: number;
  /** Colour of the ground grid's ordinary lines. */
  gridColor: number;
  /** Colour of the two grid lines crossing at the origin, which is what says where zero is. */
  gridOriginColor: number;
  /** Roughly how many cells the grid is drawn with across the level; the step itself is rounded to a readable one. */
  gridCells: number;
  /** Times each of those cells is split again, for a finer measure that still reaches the level. */
  gridSubdivision: number;
  /** Colour of the grid and box outlining the extent the level claims. */
  boundsColor: number;
  /** Colour of the disc drawn where the light comes from. */
  sunColor: number;
  /** Its width in device pixels. */
  sunSize: number;
}

export const DEFAULT_LEVEL_RENDER_CONFIG: ILevelRenderConfig = {
  backgroundColor: 0x202428,
  boundsColor: 0xffb300,
  cameraFar: 5000,
  cameraNear: 0.2,
  gridCells: 40,
  gridSubdivision: 2,
  gridColor: 0x3a4148,
  gridOriginColor: 0x7a8894,
  sunColor: 0xfff2c8,
  sunSize: 24,
};
