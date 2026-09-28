import { ERendererCameraController } from "#/contract/renderer-camera";
import { IRendererCameraLens } from "#/contract/renderer-camera-lens";
import { TRendererVector } from "#/contract/renderer-vector";

/**
 * A camera flown through a scene.
 * Described again from the same start, it keeps where it has flown and takes only the rest.
 */
export interface IRendererFlyCamera extends IRendererCameraLens {
  kind: ERendererCameraController.FLY;
  /** Where the camera starts, and returns to on reset. */
  position: TRendererVector;
  /** What it looks at from there. */
  target: TRendererVector;
  /** Metres a second at a walk. */
  speed: number;
  /** Times the speed while the boost key is held. */
  boost: number;
  /** Radians of turn per pixel dragged. */
  sensitivity: number;
}
