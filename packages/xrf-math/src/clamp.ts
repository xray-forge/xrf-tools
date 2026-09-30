/**
 * The value held within its bounds.
 *
 * @param value - The value.
 * @param min - The lowest it may be.
 * @param max - The highest it may be, which wins over `min` when the two cross.
 * @returns The value, or the bound it passed.
 */
export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

/**
 * The value held within zero and one, as a share, a weight or a colour channel is.
 *
 * @param value - The value.
 * @returns The value, or the bound it passed.
 */
export function saturate(value: number): number {
  return clamp(value, 0, 1);
}
