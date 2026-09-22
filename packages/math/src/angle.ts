/**
 * The angle in degrees.
 *
 * @param radians - The angle, as every trigonometric function and every engine chunk states one.
 * @returns The same angle in the unit a person reads and sets.
 */
export function toDegrees(radians: number): number {
  return (radians * 180) / Math.PI;
}

/**
 * The angle in radians.
 *
 * @param degrees - The angle, as a control offers one.
 * @returns The same angle in the unit the maths takes.
 */
export function toRadians(degrees: number): number {
  return (degrees * Math.PI) / 180;
}
