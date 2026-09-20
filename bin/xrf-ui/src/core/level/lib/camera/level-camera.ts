import { Camera, Vector3 } from "three";

import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { IXrayHeading, toXrayHeading, toXraySpace } from "@/core/render/lib/scene/render-space";

/**
 * Where the camera is and which way it faces, in the coordinates the level's own data is written in.
 */
export interface ILevelCamera extends IXrayHeading {
  position: ILevelPoint;
}

/** What a viewer shows before its camera has ever reported, which is nothing rather than a guess at the origin. */
export const UNPLACED_LEVEL_CAMERA: ILevelCamera = {
  heading: 0,
  pitch: 0,
  position: { x: 0, y: 0, z: 0 },
};

/**
 * Reads a camera's placement the way the engine would state it.
 *
 * @param camera - The camera drawing the level.
 * @param facing - Scratch vector to read the forward direction into, so a per-frame read allocates nothing.
 * @returns Where it is and where it faces, in the level's own coordinates.
 */
export function toLevelCamera(camera: Camera, facing: Vector3 = new Vector3()): ILevelCamera {
  camera.getWorldDirection(facing);

  return { ...toXrayHeading(facing), position: toXraySpace(camera.position) };
}
