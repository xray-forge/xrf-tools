/**
 * Everything about how the preview looks, as one value.
 *
 * A parameter rather than module constants so the look is adjustable without editing the scene: a settings surface, a
 * light theme, or a test that needs a known camera can hand over its own.
 */
export interface IVisualPreviewSceneConfig {
  backgroundColor: number;
  gridColor: number;
  /** Colour of the two grid lines crossing at the origin, so a model's own zero is never guessed. */
  gridOriginColor: number;
  gridCells: number;
  meshColor: number;
  /** Colour of the bind pose overlay, chosen to read against both the mesh and the background. */
  skeletonColor: number;
  /** Colour of the marker for a joint selected in the bones panel. */
  highlightColor: number;
  /** Marker size in pixels, unattenuated so it reads the same on a pistol part and on an actor. */
  highlightSize: number;
  /** Vertical field of view in degrees, which also sets how far a fitted camera has to stand back. */
  cameraFieldOfView: number;
  /** How much room to leave around a fitted model, so it does not touch the viewport edges. */
  cameraFitMargin: number;
  /**
   * Direction the camera is placed in, scaled by the fitted distance: in front of a model, which faces the engine's `+z`
   * and so renderer `-z`.
   */
  cameraDirection: [number, number, number];
  /** How many times the procedural uv checkerboard repeats across the uv range. */
  checkerRepeat: number;
}

export const DEFAULT_VISUAL_PREVIEW_SCENE_CONFIG: IVisualPreviewSceneConfig = {
  backgroundColor: 0x353535,
  gridColor: 0x505050,
  gridOriginColor: 0x787878,
  gridCells: 25,
  meshColor: 0xb0a999,
  skeletonColor: 0x4fc3f7,
  highlightColor: 0xffb300,
  highlightSize: 9,
  cameraFieldOfView: 50,
  cameraFitMargin: 1.6,
  cameraDirection: [0.6, 0.5, -0.8],
  checkerRepeat: 6,
};
