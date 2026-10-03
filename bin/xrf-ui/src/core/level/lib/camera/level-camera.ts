import { toDegrees } from "@xrf/math";

import { ILevelPoint } from "@/core/level/lib/camera/level-point";
import { IXrayHeading } from "@/core/render/lib/scene/render-space";
import { formatDegrees } from "@/lib/format/angle";

/**
 * Where the camera is and which way it faces, in the coordinates the level's own data is written in.
 */
export interface ILevelCamera extends IXrayHeading {
  position: ILevelPoint;
}

/**
 * Where the camera is, in the coordinates the level's own data is written in.
 *
 * @param camera - Where the camera is and which way it faces.
 * @returns The position, in the engine's own axis order.
 */
export function formatLevelPosition(camera: ILevelCamera): string {
  return formatLevelPoint(camera.position);
}

/**
 * A point in the coordinates the level's own data is written in, as the camera readout states one.
 *
 * @param point - The point, in the engine's own axis order.
 * @returns It, a decimal an axis.
 */
export function formatLevelPoint(point: ILevelPoint): string {
  return `x ${point.x.toFixed(1)} y ${point.y.toFixed(1)} z ${point.z.toFixed(1)}`;
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
