import { RenderCameraPose } from "@/core/ipc/types/xrf-renderer";
import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { toXrayHeading, toXraySpace } from "@/core/render/lib/scene/render-space";

/**
 * Reads where the renderer's camera is the way the engine would state it.
 *
 * @param pose - Where the camera is and the point it looks at, in renderer space.
 * @returns Where it is and where it faces, in the level's own coordinates.
 */
export function toLevelCameraReading(pose: RenderCameraPose): ILevelCamera {
  const [x = 0, y = 0, z = 0] = pose.position.map((value) => value ?? 0);
  const [tx = 0, ty = 0, tz = 0] = pose.target.map((value) => value ?? 0);

  return { ...toXrayHeading({ x: tx - x, y: ty - y, z: tz - z }), position: toXraySpace({ x, y, z }) };
}
