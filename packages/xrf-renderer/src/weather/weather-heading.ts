import { TRendererVector } from "#/contract/renderer-vector";

/**
 * `Fvector::setHP(h, p)`: the direction a heading and a pitch point along, in engine space.
 *
 * @param heading - Radians.
 * @param pitch - Radians.
 * @returns The direction.
 */
export function toWeatherHeading(heading: number, pitch: number): TRendererVector {
  return [-Math.cos(pitch) * Math.sin(heading), Math.sin(pitch), Math.cos(pitch) * Math.cos(heading)];
}
