/** How a free camera answers input, which is a setting rather than anything the camera holds. */
export interface ILevelFlyOptions {
  /** Metres a second at a walk. */
  speed: number;
  /** What holding the modifier multiplies the speed by. */
  boost: number;
  /** Radians of pitch and yaw per pixel of pointer movement. */
  sensitivity: number;
}

export const DEFAULT_LEVEL_FLY_OPTIONS: ILevelFlyOptions = {
  boost: 5,
  sensitivity: 0.002,
  speed: 12,
};
