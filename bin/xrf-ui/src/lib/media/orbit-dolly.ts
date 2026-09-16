/** One dolly step, matching the pan-zoom wheel notch so a zoom button feels the same in a scene as over a picture. */
export const DOLLY_STEP: number = 1.2;

export type TDollyPoint = readonly [number, number, number];

/**
 * Moves an orbiting camera along the line it is looking down.
 *
 * @param position - Where the camera is.
 * @param target - What it orbits, which the step moves toward or away from.
 * @param step - Multiplier on the distance; above one moves away, below one moves closer.
 * @param minDistance - Closest the camera may come, as `OrbitControls` reports it.
 * @param maxDistance - Furthest it may go, which is `Infinity` unless a scene says otherwise.
 * @returns The camera's new position, or its current one where the distance is already at the bound asked for.
 */
export function toDolliedPosition(
  position: TDollyPoint,
  target: TDollyPoint,
  step: number,
  minDistance: number = 0,
  maxDistance: number = Number.POSITIVE_INFINITY
): TDollyPoint {
  const offsetX: number = position[0] - target[0];
  const offsetY: number = position[1] - target[1];
  const offsetZ: number = position[2] - target[2];
  const distance: number = Math.hypot(offsetX, offsetY, offsetZ);

  // A camera sitting exactly on what it orbits has no direction to be moved along.
  if (distance === 0) {
    return position;
  }

  const next: number = Math.min(Math.max(distance * step, minDistance), maxDistance);
  const ratio: number = next / distance;

  return [target[0] + offsetX * ratio, target[1] + offsetY * ratio, target[2] + offsetZ * ratio];
}
