import { IHeadingPitch, toDirection, toHeadingPitch } from "#/heading-pitch";
import { IPoint3 } from "#/point";
import { TVector3 } from "#/vector";
import { wrap } from "#/wrap";

/**
 * A direction or a point the engine states, in renderer space: its `z` negated, leaving zero as zero.
 *
 * @param vector - The vector in engine space.
 * @returns The same vector in renderer space.
 */
function toRendererVector(vector: TVector3): TVector3 {
  const [x, y, z] = vector;

  // So a readout at the origin says `0.0` rather than `-0.0`.
  return [x, y, z === 0 ? 0 : -z];
}

/**
 * Turns a point the renderer holds into the point the engine would state.
 *
 * @param point - A point in renderer space.
 * @returns The same place, as the engine states it.
 */
export function toXraySpace(point: IPoint3): IPoint3 {
  // The flip is its own inverse.
  const [x, y, z] = toRendererVector([point.x, point.y, point.z]);

  return { x, y, z };
}

/**
 * The inverse of {@link toXraySpace}, for a coordinate that arrives the way the engine states it.
 *
 * @param point - A point as the engine states it.
 * @returns The same place in renderer space.
 */
export function toRendererSpace(point: IPoint3): IPoint3 {
  const [x, y, z] = toRendererVector([point.x, point.y, point.z]);

  return { x, y, z };
}

/**
 * Reads a direction the way `Fvector::getHP` does, from a direction the renderer holds.
 *
 * @param direction - Which way something faces, in renderer space; need not be normalized.
 * @returns Its heading and pitch, as the engine states them, the heading normalized to `[0, 2pi)`.
 */
export function toXrayHeading(direction: IPoint3): IHeadingPitch {
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
export function toRendererFacing(heading: IHeadingPitch): IPoint3 {
  const [x, y, z] = toDirection(heading);

  return toRendererSpace({ x, y, z });
}
