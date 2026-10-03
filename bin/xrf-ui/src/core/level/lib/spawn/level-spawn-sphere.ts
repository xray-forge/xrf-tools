import { Nullable } from "@xrf/types";

import { LevelSpawnObject } from "@/core/ipc/types/xrf-app";
import { VisualTransform } from "@/core/ipc/types/xrf-visual";
import { IRenderPoint, toXraySpace } from "@/core/render/lib/scene/render-space";

/** What an object is taken to span before its model is drawn: a metre about where it stands. */
const UNREAD_RADIUS: number = 1;

/** A sphere in renderer space. */
export interface ILevelSpawnSphere {
  center: IRenderPoint;
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

  const { c } = object.transform;

  return { center: { x: c.x ?? 0, y: c.y ?? 0, z: c.z ?? 0 }, radius: UNREAD_RADIUS };
}

/**
 * @param transform - Where an object stands, in renderer space.
 * @returns Where, as the engine states it.
 */
export function toLevelSpawnPosition(transform: VisualTransform): IRenderPoint {
  return toXraySpace({ x: transform.c.x ?? 0, y: transform.c.y ?? 0, z: transform.c.z ?? 0 });
}
