import { IRendererCameraPose } from "@xrf/renderer";

import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { toXrayHeading, toXraySpace } from "@/core/render/lib/scene/render-space";

/**
 * Reads where the renderer's camera is the way the engine would state it.
 *
 * @param pose - Where the camera is and the point it looks at, in renderer space.
 * @returns Where it is and where it faces, in the level's own coordinates.
 */
export function toLevelCamera(pose: IRendererCameraPose): ILevelCamera {
  const [x, y, z] = pose.position;
  const [tx, ty, tz] = pose.target;

  return { ...toXrayHeading({ x: tx - x, y: ty - y, z: tz - z }), position: toXraySpace({ x, y, z }) };
}
