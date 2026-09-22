import { Nullable } from "@xrf/types";

import { formatNumber } from "@/lib/format/number";

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
