/** `fWhiteIntensity` of `tonemap` (`shaders/r3/common_functions.h`). */
const WHITE_INTENSITY: number = 1.7;

/**
 * The engine's Reinhard curve, as `tonemap` applies it to one channel.
 *
 * @param value - The channel before the curve.
 * @param scale - What adaptation multiplies by first.
 * @returns The channel after it.
 */
export function toneMapReinhard(value: number, scale: number): number {
  const x: number = value * scale;

  return (x * (1 + x / (WHITE_INTENSITY * WHITE_INTENSITY))) / (x + 1);
}
