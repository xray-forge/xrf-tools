import { Nullable } from "@xrf/types";

import { LevelSpawnObject } from "@/core/ipc/types/xrf-app";
import { VisualSphere, VisualTransform } from "@/core/ipc/types/xrf-visual";
import { ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import { IRenderPoint, toXraySpace } from "@/core/render/lib/scene/render-space";

/** What an object is taken to span while its model is still being read: a metre about where it stands. */
const UNREAD_RADIUS: number = 1;

/** A sphere in renderer space. */
export interface ILevelSpawnSphere {
  center: IRenderPoint;
  radius: number;
}

/**
 * @param object - A spawned object.
 * @param model - The model it stands as, or null while it is read.
 * @returns What it spans where it stands, in renderer space: its visual's sphere as the header declares it, stood in
 *   its place, or a metre about its place without a model.
 */
export function toLevelSpawnSphere(object: LevelSpawnObject, model: Nullable<ILevelSpawnModel>): ILevelSpawnSphere {
  const sphere: Nullable<VisualSphere> = model?.description.description.declaredBounds.boundingSphere ?? null;
  const { i, j, k, c } = object.transform;
  const [x, y, z] = [sphere?.center.x ?? 0, sphere?.center.y ?? 0, sphere?.center.z ?? 0];

  return {
    center: {
      x: (c.x ?? 0) + (i.x ?? 0) * x + (j.x ?? 0) * y + (k.x ?? 0) * z,
      y: (c.y ?? 0) + (i.y ?? 0) * x + (j.y ?? 0) * y + (k.y ?? 0) * z,
      z: (c.z ?? 0) + (i.z ?? 0) * x + (j.z ?? 0) * y + (k.z ?? 0) * z,
    },
    radius: sphere?.radius || UNREAD_RADIUS,
  };
}

/**
 * @param transform - Where an object stands, in renderer space.
 * @returns Where, as the engine states it.
 */
export function toLevelSpawnPosition(transform: VisualTransform): IRenderPoint {
  return toXraySpace({ x: transform.c.x ?? 0, y: transform.c.y ?? 0, z: transform.c.z ?? 0 });
}
