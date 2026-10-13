import { Nullable } from "@xrf/types";

import { formatNumber } from "#/number";

/**
 * Formats a measurement already taken in seconds, keeping an absent one visibly absent.
 *
 * @param seconds - Seconds to render, possibly absent.
 * @returns The measurement with its unit, or the placeholder.
 */
export function formatSeconds(seconds: Nullable<number>): string {
  return `${formatNumber(seconds, 2)} s`;
}

/**
 * Formats a millisecond duration for reports, in the largest unit its rounded value reaches: whole milliseconds, then
 * tenths of a second, then minutes and whole seconds, then hours and minutes. A value is rounded to a unit's precision
 * before the unit is chosen, so `59.96 s` reads `1 m 0 s`, never `60.0 s`.
 *
 * @param durationMs - Duration in milliseconds.
 * @returns The duration in milliseconds, seconds, minutes and seconds, or hours and minutes.
 */
export function formatDuration(durationMs: number): string {
  const milliseconds: number = Math.round(durationMs);

  if (milliseconds < 1000) {
    return `${milliseconds} ms`;
  }

  const tenths: number = Math.round(durationMs / 100);

  if (tenths < 600) {
    return `${(tenths / 10).toFixed(1)} s`;
  }

  const seconds: number = Math.round(durationMs / 1000);
  const minutes: number = Math.floor(seconds / 60);

  if (minutes < 60) {
    return `${minutes} m ${seconds - minutes * 60} s`;
  }

  const hours: number = Math.floor(minutes / 60);

  return `${hours} h ${minutes - hours * 60} m`;
}

/**
 * Formats a short span, a frame's or a stage's, to a tenth of a millisecond: what `formatDuration` rounds away.
 *
 * @param durationMs - Duration in milliseconds.
 * @returns The duration in milliseconds.
 */
export function formatMilliseconds(durationMs: number): string {
  return `${durationMs.toFixed(1)} ms`;
}

/**
 * Formats a playback position as `mm:ss`.
 *
 * @param seconds - Position or duration in seconds, as a media element reports it.
 * @returns Zero padded `mm:ss`, falling back to `00:00` for the `NaN` an unloaded element reports.
 */
export function formatPlaybackTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) {
    return "00:00";
  }

  const whole: number = Math.floor(seconds);

  return `${String(Math.floor(whole / 60)).padStart(2, "0")}:${String(whole % 60).padStart(2, "0")}`;
}
