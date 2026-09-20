/**
 * A point in one of the two spaces an X-Ray asset is ever in.
 */
export interface IRenderPoint {
  x: number;
  y: number;
  z: number;
}

/**
 * Turns a point the renderer holds into the point the engine would state.
 *
 * @param point - A point in renderer space.
 * @returns The same place, as the engine states it.
 */
export function toXraySpace(point: IRenderPoint): IRenderPoint {
  return { x: point.x, y: point.y, z: negate(point.z) };
}

/**
 * The inverse of {@link toXraySpace}, for a coordinate that arrives the way the engine states it.
 *
 * @param point - A point as the engine states it.
 * @returns The same place in renderer space.
 */
export function toRendererSpace(point: IRenderPoint): IRenderPoint {
  return { x: point.x, y: point.y, z: negate(point.z) };
}

/** Negation that leaves zero alone, so a readout at the origin says `0.0` rather than `-0.0`. */
function negate(value: number): number {
  return value === 0 ? 0 : -value;
}
