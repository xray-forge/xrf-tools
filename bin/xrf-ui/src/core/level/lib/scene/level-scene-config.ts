/**
 * Everything about how a level preview is framed, as one value.
 */
export interface ILevelPreviewSceneConfig {
  backgroundColor: number;
  /** Vertical field of view in degrees. */
  cameraFieldOfView: number;
  /** How far the camera sees. */
  cameraNear: number;
  cameraFar: number;
  /** Colour of the ground grid's ordinary lines. */
  gridColor: number;
  /** Colour of the two grid lines crossing at the origin, which is what says where zero is. */
  gridOriginColor: number;
  /** Roughly how many cells the grid is drawn with across the level; the step itself is rounded to a readable one. */
  gridCells: number;
  /** Colour of the box outlining the extent the level claims. */
  boundsColor: number;
}

export const DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG: ILevelPreviewSceneConfig = {
  backgroundColor: 0x202428,
  cameraFar: 5000,
  cameraFieldOfView: 65,
  gridCells: 40,
  gridColor: 0x3a4148,
  gridOriginColor: 0x7a8894,
  boundsColor: 0xffb300,
  cameraNear: 0.1,
};
