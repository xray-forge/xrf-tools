/** Seconds a day lasts. */
export const LEVEL_WEATHER_DAY: number = 86_400;

/**
 * @param seconds - Seconds since midnight.
 * @param isPrecise - Whether the seconds are said too.
 * @returns The time of day as a keyframe names one, `HH:MM` or `HH:MM:SS`.
 */
export function formatLevelWeatherTime(seconds: number, isPrecise: boolean = false): string {
  const whole: number = Math.floor(((seconds % LEVEL_WEATHER_DAY) + LEVEL_WEATHER_DAY) % LEVEL_WEATHER_DAY);
  const parts: Array<number> = [Math.floor(whole / 3600), Math.floor(whole / 60) % 60];

  if (isPrecise) {
    parts.push(whole % 60);
  }

  return parts.map((part: number) => String(part).padStart(2, "0")).join(":");
}
