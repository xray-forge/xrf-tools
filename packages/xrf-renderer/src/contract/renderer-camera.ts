import { IRendererFlyCamera } from "#/contract/renderer-fly-camera";
import { IRendererOrbitCamera } from "#/contract/renderer-orbit-camera";

/**
 * How the camera is driven.
 */
export enum ERendererCameraController {
  /** Orbits a target under the pointer, as a model or texture preview does. */
  ORBIT = "orbit",
  /** Flies free, turned by a drag and moved by the keys, as a level is walked. */
  FLY = "fly",
}

/** Every camera a consumer can ask for. */
export type TRendererCamera = IRendererOrbitCamera | IRendererFlyCamera;
