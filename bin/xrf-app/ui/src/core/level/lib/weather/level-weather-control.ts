import { clamp } from "@xrf/math";

/** Seconds from midnight to noon. */
export const LEVEL_WEATHER_NOON: number = 12 * 60 * 60;

/** The engine's own bounds on its time factor. */
export const LEVEL_WEATHER_FACTOR_LIMITS = { max: 1000, min: 1 } as const;

/**
 * How the level's weather plays.
 */
export interface ILevelWeatherControl {
  /** Game seconds a real second, the engine's time factor. */
  factor: number;
  isPaused: boolean;
  /** Whether a vanilla weather stands the sun astronomically rather than by its keyframes. */
  isDynamicSun: boolean;
}

/** Paused, at the engine's own time factor, with the sun the keyframes stand. */
export const DEFAULT_LEVEL_WEATHER_CONTROL: ILevelWeatherControl = {
  factor: 12,
  isDynamicSun: false,
  isPaused: true,
};

/**
 * @param factor - Game seconds a real second, as asked for.
 * @returns The same in whole seconds, within the engine's bounds.
 */
export function toLevelWeatherFactor(factor: number): number {
  return clamp(Math.round(factor), LEVEL_WEATHER_FACTOR_LIMITS.min, LEVEL_WEATHER_FACTOR_LIMITS.max);
}
