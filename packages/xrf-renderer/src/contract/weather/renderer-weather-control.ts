import { Nullable } from "@xrf/types";

/**
 * How the renderer plays its weather: the clock and the view's own switches.
 */
export interface IRendererWeatherControl {
  /** Seconds since midnight to play from, or null to play on from where the clock stands. */
  time: Nullable<number>;
  /** Game seconds a real second, the engine's time factor. */
  factor: number;
  isPaused: boolean;
  /** Whether a vanilla weather stands the sun astronomically rather than by its keyframes. */
  isDynamicSun: boolean;
  isFogged: boolean;
  isWindy: boolean;
  /** Whether the clouds are drawn over the sky. */
  isClouded: boolean;
  /** Whether it rains where the weather rains. */
  isRainy: boolean;
  /** Whether bolts strike where the weather strikes them. */
  isThundering: boolean;
}
