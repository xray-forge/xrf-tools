/**
 * Everything about how a level preview looks, as one value.
 */
export interface ILevelPreviewSceneConfig {
  backgroundColor: number;
  /** Vertical field of view in degrees. */
  cameraFieldOfView: number;
  /** How far the camera sees. */
  cameraNear: number;
  cameraFar: number;
  /** Fraction of a level's radius the camera stands back by when it first frames one. */
  cameraFitMargin: number;
  ambientIntensity: number;
  sunIntensity: number;
  /** Where the directional light comes from, which is only a stand-in until levels light themselves. */
  sunDirection: [number, number, number];
}

export const DEFAULT_LEVEL_PREVIEW_SCENE_CONFIG: ILevelPreviewSceneConfig = {
  ambientIntensity: 1.1,
  backgroundColor: 0x202428,
  cameraFar: 5000,
  cameraFieldOfView: 65,
  cameraFitMargin: 0.6,
  cameraNear: 0.1,
  sunDirection: [0.5, 1, 0.35],
  sunIntensity: 1.8,
};
