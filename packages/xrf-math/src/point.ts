import { Maybe, Nullable } from "@xrf/types";

/** A point or a direction with named axes, in whichever space its holder states. */
export interface IPoint3 {
  x: number;
  y: number;
  z: number;
}

/**
 * A point from the wire, where an axis may be null because a non-finite float serialises to null.
 *
 * @param vector - The point as received, or nothing.
 * @returns The point, an absent axis or an absent point read as zero.
 */
export function toPoint3(vector: Maybe<{ x: Nullable<number>; y: Nullable<number>; z: Nullable<number> }>): IPoint3 {
  return { x: vector?.x ?? 0, y: vector?.y ?? 0, z: vector?.z ?? 0 };
}
