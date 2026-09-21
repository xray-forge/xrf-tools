import { EMPTY_LEVEL_FLY_INPUT, ILevelFlyInput } from "@/core/level/lib/camera/level-fly-input";

/**
 * What the person did to the camera since the last frame.
 */
export interface ILevelFlyMotion {
  /** Pointer movement across since the last drain, in pixels. */
  lookX: number;
  /** Pointer movement down since the last drain, in pixels. */
  lookY: number;
  /** Keys held at the moment of the drain. */
  keys: ILevelFlyInput;
}

/** Nobody is doing anything, which is what an unfocused viewport reports. */
export const EMPTY_LEVEL_FLY_MOTION: ILevelFlyMotion = { keys: EMPTY_LEVEL_FLY_INPUT, lookX: 0, lookY: 0 };

/**
 * Where a frame reads what the person is doing.
 */
export interface ILevelMotionSource {
  /**
   * @returns What has happened since the last call, which is then forgotten: a look is a movement and counting
   *   one twice turns the camera twice as far.
   */
  drain(): ILevelFlyMotion;
}
