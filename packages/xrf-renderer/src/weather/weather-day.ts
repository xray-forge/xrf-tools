/** Seconds a day lasts, `DAY_LENGTH`. */
export const WEATHER_DAY_LENGTH: number = 86_400;

/**
 * @param time - Seconds, any number of days either way.
 * @returns The time of day it falls at, in seconds since midnight.
 */
export function toWeatherTimeOfDay(time: number): number {
  return ((time % WEATHER_DAY_LENGTH) + WEATHER_DAY_LENGTH) % WEATHER_DAY_LENGTH;
}
