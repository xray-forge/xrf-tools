/** The sign every angle in this application is written with, so no surface spells it for itself. */
export const DEGREE_SIGN: string = "°";

/**
 * Formats an angle in degrees.
 *
 * @param value - The angle, in degrees.
 * @param digits - Fixed decimal places to render, none by default since an angle a person sets is a whole number.
 * @returns The angle with its sign, ready to read.
 */
export function formatDegrees(value: number, digits: number = 0): string {
  return `${value.toFixed(digits)}${DEGREE_SIGN}`;
}
