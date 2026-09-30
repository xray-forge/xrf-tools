import { toDirection, toHeadingPitch, wrap } from "@xrf/math";
import { toEngineVector, toRendererVector } from "@xrf/renderer";
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
  const [x, y, z] = toEngineVector([point.x, point.y, point.z]);

  return { x, y, z };
}

/**
 * The inverse of {@link toXraySpace}, for a coordinate that arrives the way the engine states it.
 *
 * @param point - A point as the engine states it.
 * @returns The same place in renderer space.
 */
export function toRendererSpace(point: IRenderPoint): IRenderPoint {
  const [x, y, z] = toRendererVector([point.x, point.y, point.z]);

  return { x, y, z };
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
  const { x, y, z } = toXraySpace(direction);
  const { heading, pitch } = toHeadingPitch([x, y, z]);

  // Counted from zero round one turn, so a bearing never arrives negative.
  return { heading: wrap(heading, Math.PI * 2), pitch };
}

/**
 * Builds a direction the way `Fvector::setHP` does, the inverse of {@link toXrayHeading}.
 *
 * @param heading - Where something faces, as the engine states it.
 * @returns Which way it faces, normalized, in renderer space.
 */
export function toRendererFacing(heading: IXrayHeading): IRenderPoint {
  const [x, y, z] = toDirection(heading);

  return toRendererSpace({ x, y, z });
}
