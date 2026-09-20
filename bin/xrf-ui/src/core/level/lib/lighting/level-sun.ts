import { Vector3d } from "@/core/ipc/types/xrf-math";
import { toRendererSpace } from "@/core/render/lib/scene/render-space";
import { toDegrees } from "@/lib/math/angle";
import { Nullable } from "@/lib/types/general";

/**
 * The elevation and azimuth a level's own sun comes to, for a viewer offering to light it the way xrLC did.
 *
 * @param direction - The direction the level's sun light travels, in the level's own axes.
 * @returns The angles, or null when there is no direction to take them from.
 */
export function toSunAngles(direction: Nullable<Vector3d>): Nullable<{ elevation: number; azimuth: number }> {
  if (!direction) {
    return null;
  }

  // The wire admits a null component for a vector a file left short; a sun aimed at nothing is no sun.
  const { x, y, z } = toRendererSpace({ x: direction.x ?? 0, y: direction.y ?? 0, z: direction.z ?? 0 });
  const length: number = Math.sqrt(x * x + y * y + z * z);

  if (!length) {
    return null;
  }

  // Opposite the travel, because this describes where the sun is rather than where its light goes.
  const toSun = { x: -x / length, y: -y / length, z: -z / length };

  return {
    azimuth: toDegrees(Math.atan2(toSun.x, toSun.z)),
    elevation: toDegrees(Math.asin(Math.max(-1, Math.min(1, toSun.y)))),
  };
}
