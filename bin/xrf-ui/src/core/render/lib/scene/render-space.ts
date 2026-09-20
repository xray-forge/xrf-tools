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

/**
 * Where something faces, as the engine states it.
 */
export interface IXrayHeading {
  /** Radians about the vertical axis, zero facing `+z` and turning towards `-x`, normalized to `[0, 2pi)`. */
  heading: number;
  /** Radians away from the horizon, positive above it, in `[-pi/2, pi/2]`. */
  pitch: number;
}

/**
 * Reads a direction the way `Fvector::getHP` does, from a direction the renderer holds.
 *
 * @param direction - Which way something faces, in renderer space; need not be normalized.
 * @returns Its heading and pitch, as the engine states them.
 */
export function toXrayHeading(direction: IRenderPoint): IXrayHeading {
  const facing: IRenderPoint = toXraySpace(direction);
  const flat: number = Math.hypot(facing.x, facing.z);

  return {
    // The engine measures its heading the other way round from `atan2`, which is the whole content of `getHP`. A
    // direction straight up or down has no heading to read, and `getHP` answers zero for it rather than guessing.
    heading: flat === 0 ? 0 : toTurn(-Math.atan2(facing.x, facing.z)),
    pitch: Math.atan2(facing.y, flat),
  };
}

/** The same angle counted from zero round one turn, so a bearing never arrives negative. */
function toTurn(radians: number): number {
  const turn: number = Math.PI * 2;

  return ((radians % turn) + turn) % turn;
}
