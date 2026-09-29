/**
 * What a keyframe set by hand was seeded from: the cycle playing, and the time its clock stood at.
 */
export interface ILevelWeatherSeed {
  cycle: string;
  /** Seconds since midnight. */
  time: number;
}
