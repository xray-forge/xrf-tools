/**
 * Everything about how the preview looks, as one value.
 *
 * A parameter rather than module constants so the look is adjustable without editing the scene: a settings surface, a
 * light theme, or a test that needs a known camera can hand over its own. The scene reads it once at construction,
 * which is enough while the values are chosen per scene rather than changed live.
 */
export interface IVisualPreviewSceneConfig {
  backgroundColor: number;
  gridColor: number;
  meshColor: number;
  /** Colour of the bind pose overlay, chosen to read against both the mesh and the background. */
  skeletonColor: number;
  /** Vertical field of view in degrees, which also sets how far a fitted camera has to stand back. */
  cameraFieldOfView: number;
  /** How much room to leave around a fitted model, so it does not touch the viewport edges. */
  cameraFitMargin: number;
  /** Direction the camera is placed in, scaled by the fitted distance. */
  cameraDirection: [number, number, number];
  /** Side length in pixels of the procedural uv checkerboard. */
  checkerSize: number;
  /** How many times that checkerboard repeats across the uv range. */
  checkerRepeat: number;
}

export const DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG: IVisualPreviewSceneConfig = {
  backgroundColor: 0x353535,
  gridColor: 0x505050,
  meshColor: 0xb0a999,
  skeletonColor: 0x4fc3f7,
  cameraFieldOfView: 50,
  cameraFitMargin: 1.6,
  cameraDirection: [0.6, 0.5, 0.8],
  checkerSize: 8,
  checkerRepeat: 6,
};
