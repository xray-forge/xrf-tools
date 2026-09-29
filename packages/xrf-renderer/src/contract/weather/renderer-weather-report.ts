/**
 * Where the renderer's weather stands.
 */
export interface IRendererWeatherReport {
  /** Seconds since midnight. */
  time: number;
  /** The keyframes either side, by index. */
  keyframes: readonly [number, number];
  /** How far from the first to the second. */
  weight: number;
}
