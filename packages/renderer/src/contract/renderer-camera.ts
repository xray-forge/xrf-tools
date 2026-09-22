import { TRendererVector } from "#/contract/renderer-lighting";

/**
 * How the camera is driven.
 */
export enum ERendererCameraController {
  /** Orbits a target under the pointer, as a model or texture preview does. */
  ORBIT = "orbit",
}

/**
 * A camera orbiting a target.
 */
export interface IRendererOrbitCamera {
  kind: ERendererCameraController.ORBIT;
  /** What the camera looks at and turns around. */
  target: TRendererVector;
  /** Where the camera starts, and returns to on reset. */
  position: TRendererVector;
  /** Vertical field of view, in degrees. */
  fieldOfView: number;
  near: number;
  far: number;
}

/** Every camera a consumer can ask for. */
export type TRendererCamera = IRendererOrbitCamera;

/**
 * What a consumer can ask of the camera it chose.
 */
export enum ERendererCameraCommand {
  /** Back to where the camera started. */
  RESET = "reset",
  /** Towards the target or away from it, by a multiplier on the distance: above one moves away. */
  DOLLY = "dolly",
}

/** A command, with what it needs. */
export type TRendererCameraCommand =
  { kind: ERendererCameraCommand.RESET } | { kind: ERendererCameraCommand.DOLLY; step: number };

/** Where the camera is, as a report states it. */
export interface IRendererCameraPose {
  position: TRendererVector;
  /** The point it looks at. */
  target: TRendererVector;
}
