import { Vector3d } from "@/core/ipc/types/xrf-math";
import { ILevelLighting } from "@/core/level/lib/lighting/level-lighting";
import { toRendererSpace } from "@/core/render/lib/render-space";
import { Nullable } from "@/lib/types/general";

/** Degrees in a half turn, for the conversions below. */
const HALF_TURN: number = 180;

/**
 * Where the light comes from, in the renderer's own axes.
 *
 * @param lighting - The lighting to read.
 * @param distance - How far out to put it, which only has to clear the level.
 * @returns The position, ready to assign.
 */
export function toSunPosition(lighting: ILevelLighting, distance: number): [number, number, number] {
  const elevation: number = (lighting.sunElevation * Math.PI) / HALF_TURN;
  const azimuth: number = (lighting.sunAzimuth * Math.PI) / HALF_TURN;
  const horizontal: number = Math.cos(elevation) * distance;

  return [horizontal * Math.sin(azimuth), Math.sin(elevation) * distance, horizontal * Math.cos(azimuth)];
}

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
    azimuth: (Math.atan2(toSun.x, toSun.z) * HALF_TURN) / Math.PI,
    elevation: (Math.asin(Math.max(-1, Math.min(1, toSun.y))) * HALF_TURN) / Math.PI,
  };
}
