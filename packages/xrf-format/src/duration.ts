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
 * Formats a millisecond duration for reports.
 *
 * The previous result components printed `duration / 1000` straight into the DOM, which produced
 * values like `0.123 sec` for a fast run and `98.4315 sec` for a slow one.
 *
 * @param durationMs - Duration in milliseconds.
 * @returns The duration in milliseconds, seconds, or minutes and seconds.
 */
export function formatDuration(durationMs: number): string {
  if (durationMs < 1000) {
    return `${Math.round(durationMs)} ms`;
  }

  const seconds: number = durationMs / 1000;

  if (seconds < 60) {
    return `${seconds.toFixed(1)} s`;
  }

  const minutes: number = Math.floor(seconds / 60);

  if (minutes < 60) {
    return `${minutes} m ${Math.round(seconds - minutes * 60)} s`;
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
