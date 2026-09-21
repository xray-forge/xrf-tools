import { Camera, Vector3 } from "three";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { toXrayHeading, toXraySpace } from "@/core/render/lib/scene/render-space";

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
