import { ERendererCameraController } from "#/contract/renderer-camera";
import { IRendererCameraLens } from "#/contract/renderer-camera-lens";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * A camera orbiting a target.
 * Described again from the same start, it keeps where it has been turned and takes only the lens.
 */
export interface IRendererOrbitCamera extends IRendererCameraLens {
  kind: ERendererCameraController.ORBIT;
  /** What the camera looks at and turns around. */
  target: TRendererVector;
  /** Where the camera starts, and returns to on reset. */
  position: TRendererVector;
}
