import { IPoint3, toPoint3, toXraySpace } from "@xrf/math";
import { Nullable } from "@xrf/types";

import { LevelSpawnObject } from "@/core/ipc/types/xrf-app-levels";
import { VisualTransform } from "@/core/ipc/types/xrf-visual";

/** What an object is taken to span before its model is drawn: a metre about where it stands. */
const UNREAD_RADIUS: number = 1;

/** A sphere in renderer space. */
export interface ILevelSpawnSphere {
  center: IPoint3;
  radius: number;
}

/**
 * @param object - A spawned object.
 * @param sphere - The sphere its model spans where it stands, in renderer space, centre then radius, as the renderer
 *   holds it; null before its model is drawn.
 * @returns What it spans, or a metre about where it stands without a model.
 */
export function toLevelSpawnSphere(
  object: LevelSpawnObject,
  sphere: Nullable<readonly [number, number, number, number]>
): ILevelSpawnSphere {
  if (sphere) {
    return { center: { x: sphere[0], y: sphere[1], z: sphere[2] }, radius: sphere[3] || UNREAD_RADIUS };
  }

  return { center: toPoint3(object.transform.c), radius: UNREAD_RADIUS };
}

/**
 * @param transform - Where an object stands, in renderer space.
 * @returns Where, as the engine states it.
 */
export function toLevelSpawnPosition(transform: VisualTransform): IPoint3 {
  return toXraySpace(toPoint3(transform.c));
}
