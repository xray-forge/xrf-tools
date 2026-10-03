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

/** One dolly step, matching the pan-zoom wheel notch so a zoom button feels the same in a scene as over a picture. */
export const DOLLY_STEP: number = 1.2;
