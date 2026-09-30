import { EPS_S } from "#/math-constants";

/** Where a direction points, as `Fvector::getHP` reads it and `Fvector::setHP` takes it, in radians. */
export interface IHeadingPitch {
  /** About the vertical, zero along `+z` and turning towards `-x`. */
  heading: number;
  /** Up from the horizontal. */
  pitch: number;
}

/**
 * `Fvector::setHP`.
 *
 * @param angles - Where the direction points.
 * @returns The direction, of length one, in engine space.
 */
export function toDirection(angles: IHeadingPitch): [number, number, number] {
  const { heading, pitch } = angles;

  return [-Math.cos(pitch) * Math.sin(heading), Math.sin(pitch), Math.cos(pitch) * Math.cos(heading)];
}

/**
 * `Fvector::getHP`: a direction straight up or down has no heading, and reads as zero.
 *
 * @param direction - A direction in engine space, of any length.
 * @returns Where it points, the heading in `(-pi, pi]`.
 */
export function toHeadingPitch(direction: readonly [number, number, number]): IHeadingPitch {
  const [x, y, z] = direction;
  const flat: number = Math.hypot(x, z);

  return {
    heading: Math.abs(x) < EPS_S && Math.abs(z) < EPS_S ? 0 : Math.atan2(-x, z),
    pitch: Math.atan2(y, flat),
  };
}
