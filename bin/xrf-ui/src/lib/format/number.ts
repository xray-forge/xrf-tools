import { Nullable } from "@xrf/types";

/** Shown in place of a value that is absent or not a number, rather than a misleading `0`. */
export const ABSENT_VALUE: string = "—";

/**
 * Formats a number for display, keeping an absent or non-finite value visibly absent.
 *
 * A rust `f32` crosses the ipc boundary as `number | null`, because a non-finite float serialises to null, and such
 * values do occur in game data. Rendering those as `0` would state a measurement that was never taken.
 *
 * Non-finite values take the placeholder too: `toFixed` renders them as `NaN` or `Infinity`, which reads as a measured
 * result rather than as the absence of one.
 *
 * @param value - Value to format, possibly absent.
 * @param digits - Fixed decimal places to render.
 * @param fallback - Shown instead when there is no number to render, for a caller whose surface needs its own wording.
 * @returns The formatted value, or the fallback when there is nothing to show.
 */
export function formatNumber(value: Nullable<number>, digits: number, fallback: string = ABSENT_VALUE): string {
  return value === null || !Number.isFinite(value) ? fallback : value.toFixed(digits);
}

/**
 * Formats a fraction as a percentage.
 *
 * @param value - The fraction, where `1` is the whole.
 * @param digits - Fixed decimal places to render, none by default.
 * @returns The percentage with its sign.
 */
export function formatPercent(value: number, digits: number = 0): string {
  return `${(value * 100).toFixed(digits)}%`;
}
