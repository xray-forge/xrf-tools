import { toDegrees } from "@xrf/math";

import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { IXrayHeading } from "@/core/render/lib/scene/render-space";
import { formatDegrees } from "@/lib/format/angle";

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
 * Where the camera is, in the coordinates the level's own data is written in.
 *
 * @param camera - Where the camera is and which way it faces.
 * @returns The position, in the engine's own axis order.
 */
export function formatLevelPosition(camera: ILevelCamera): string {
  const { position } = camera;

  return `x ${position.x.toFixed(1)} y ${position.y.toFixed(1)} z ${position.z.toFixed(1)}`;
}

/**
 * Which way the camera faces, as the engine states a direction.
 *
 * @param camera - Where the camera is and which way it faces.
 * @returns Its heading and pitch, in degrees.
 */
export function formatLevelFacing(camera: ILevelCamera): string {
  return `h ${formatDegrees(toDegrees(camera.heading), 1)} p ${formatDegrees(toDegrees(camera.pitch), 1)}`;
}
