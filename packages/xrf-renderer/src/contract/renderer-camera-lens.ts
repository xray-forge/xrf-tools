/**
 * What a camera sees through.
 */
export interface IRendererCameraLens {
  /** Vertical field of view, in degrees. */
  fieldOfView: number;
  near: number;
  far: number;
}
