/**
 * A time the weather was sent to, one object a seek, so seeking to the same time twice still sends it.
 */
export interface ILevelWeatherSeek {
  /** Seconds since midnight. */
  time: number;
}
