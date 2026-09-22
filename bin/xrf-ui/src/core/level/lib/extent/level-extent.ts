import { Nullable } from "@xrf/types";
import { Box3, Vector3 } from "three";

import { VisualBounds } from "@/core/ipc/types/xrf-visual";

/** What an empty box answers for a reach, so a grid built from one is still a grid rather than a point. */
const EMPTY_REACH: number = 1;

/**
 * The measured extent as a box.
 *
 * @param bounds - What the backend measured, or null for a level that reports none and for no level at all.
 * @returns The box, empty when there is nothing to measure.
 */
export function toLevelBox(bounds: Nullable<VisualBounds>): Box3 {
  if (!bounds) {
    return new Box3();
  }

  const { min, max } = bounds.boundingBox;

  return new Box3(new Vector3(min.x ?? 0, min.y ?? 0, min.z ?? 0), new Vector3(max.x ?? 0, max.y ?? 0, max.z ?? 0));
}

/**
 * How far something centred on a box has to reach to cover it, which is its longer horizontal half.
 *
 * @param box - The extent.
 * @returns The reach, in the scene's own unit.
 */
export function toBoxReach(box: Box3): number {
  if (box.isEmpty()) {
    return EMPTY_REACH;
  }

  return Math.max(box.max.x - box.min.x, box.max.z - box.min.z) / 2;
}

/**
 * How far something centred on the origin has to reach to cover a box, which is its furthest corner.
 *
 * Not the same question as {@link toBoxReach}: a level lying a kilometre off the origin needs something that reaches
 * the level, not something the size of it.
 *
 * @param box - The extent.
 * @returns The reach, in the scene's own unit.
 */
export function toOriginReach(box: Box3): number {
  if (box.isEmpty()) {
    return EMPTY_REACH;
  }

  return Math.max(Math.abs(box.min.x), Math.abs(box.max.x), Math.abs(box.min.z), Math.abs(box.max.z));
}

/**
 * The middle of a box's floor, which is where anything lying flat under it sits.
 *
 * @param box - The extent.
 * @returns The point, the origin for an empty box.
 */
export function toBoxFloor(box: Box3): Vector3 {
  if (box.isEmpty()) {
    return new Vector3();
  }

  return new Vector3((box.min.x + box.max.x) / 2, box.min.y, (box.min.z + box.max.z) / 2);
}
