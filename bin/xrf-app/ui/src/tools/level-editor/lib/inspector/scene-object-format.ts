import { ABSENT_VALUE, formatCount, formatDegrees, formatNumber } from "@xrf/format";
import {
  ESceneLightType,
  ESceneShapeElement,
  ESceneShapeUsage,
  ESceneWayType,
  SceneLightType,
  SceneShape,
  SceneShapeElement,
  SceneShapeUsage,
  SceneWayType,
} from "@xrf/ipc/types/xrf-app-levels";
import { Vector3d } from "@xrf/ipc/types/xrf-math";
import { IPoint3, toDegrees, toPoint3 } from "@xrf/math";
import { assertExhaustive, Nullable } from "@xrf/types";

import { formatLevelPoint } from "@/core/level/lib/camera/level-camera";

/**
 * How many entries of a list the Inspector writes out before it counts the rest: a way or a sector holds thousands,
 * which no reader scrolls through and a panel should not lay out.
 */
export const SCENE_LIST_LIMIT: number = 20;

/**
 * @param vector - A point in the scene's own space.
 * @returns It as the Inspector writes a place, a decimal an axis.
 */
export function formatScenePoint(vector: Vector3d): string {
  return formatLevelPoint(toPoint3(vector));
}

/**
 * @param rotation - Angles about `x`, `y` and `z`, in radians.
 * @returns Them in degrees, a decimal each.
 */
export function formatSceneRotation(rotation: Vector3d): string {
  const { x, y, z }: IPoint3 = toPoint3(rotation);

  return `x ${formatDegrees(toDegrees(x), 1)} y ${formatDegrees(toDegrees(y), 1)} z ${formatDegrees(toDegrees(z), 1)}`;
}

/**
 * @param rotation - Heading, pitch and bank, in radians, as the editor's camera keeps them.
 * @returns Them in degrees, a decimal each.
 */
export function formatSceneFacing(rotation: Vector3d): string {
  const { x, y, z }: IPoint3 = toPoint3(rotation);

  return `h ${formatDegrees(toDegrees(x), 1)} p ${formatDegrees(toDegrees(y), 1)} b ${formatDegrees(toDegrees(z), 1)}`;
}

/**
 * @param scale - A scale along each axis.
 * @returns It, two decimals an axis.
 */
export function formatSceneScale(scale: Vector3d): string {
  return `x ${formatNumber(scale.x, 2)} y ${formatNumber(scale.y, 2)} z ${formatNumber(scale.z, 2)}`;
}

/**
 * @param value - A length in metres, or none where it is not a number.
 * @returns It, two decimals and its unit, or the absent mark.
 */
export function formatSceneMetres(value: Nullable<number>): string {
  return value === null || !Number.isFinite(value) ? ABSENT_VALUE : `${formatNumber(value, 2)} m`;
}

/**
 * @param channels - A colour's channels, as many as it keeps.
 * @param digits - Decimals each channel is written with: none for bytes.
 * @returns Its channels in order, apart.
 */
export function formatSceneColor(channels: ReadonlyArray<Nullable<number>>, digits: number = 2): string {
  return channels.map((channel: Nullable<number>) => formatNumber(channel, digits)).join(" ");
}

/**
 * Writes a list out one entry a line, the entries past {@link SCENE_LIST_LIMIT} counted rather than described.
 *
 * @param items - The list.
 * @param describe - How one entry is written, given its place in the list.
 * @param limit - How many entries are written out.
 * @returns The lines, or `None` for an empty list.
 */
export function formatSceneList<T>(
  items: ReadonlyArray<T>,
  describe: (item: T, index: number) => string,
  limit: number = SCENE_LIST_LIMIT
): string {
  if (!items.length) {
    return "None";
  }

  const lines: Array<string> = items.slice(0, limit).map(describe);

  return items.length > limit
    ? [...lines, `and ${formatCount(items.length - limit)} more`].join("\n")
    : lines.join("\n");
}

/**
 * @param lightType - What kind of light a light is.
 * @returns It as the Inspector names it; a type the editor gives no name by its number.
 */
export function describeSceneLightType(lightType: SceneLightType): string {
  switch (lightType.kind) {
    case ESceneLightType.POINT:
      return "Point";

    case ESceneLightType.SPOT:
      return "Spot";

    case ESceneLightType.DIRECTIONAL:
      return "Directional";

    case ESceneLightType.UNKNOWN:
      return `Type ${lightType.value}`;

    default:
      return assertExhaustive(lightType);
  }
}

/**
 * @param usage - What a shape is used for.
 * @returns It as the Inspector names it; a usage the editor gives no name by its number.
 */
export function describeSceneShapeUsage(usage: SceneShapeUsage): string {
  switch (usage.kind) {
    case ESceneShapeUsage.COMMON:
      return "Common";

    case ESceneShapeUsage.LEVEL_BOUND:
      return "Level bound";

    case ESceneShapeUsage.UNKNOWN:
      return `Type ${usage.value}`;

    default:
      return assertExhaustive(usage);
  }
}

/**
 * @param wayType - What a way is.
 * @returns It as the Inspector names it; a type the editor gives no name by its number.
 */
export function describeSceneWayType(wayType: SceneWayType): string {
  switch (wayType.kind) {
    case ESceneWayType.PATROL_PATH:
      return "Patrol path";

    case ESceneWayType.UNKNOWN:
      return `Type ${wayType.value}`;

    default:
      return assertExhaustive(wayType);
  }
}

/**
 * @param element - One sphere or box of a shape, in its object's own space.
 * @returns Its kind, size and where it stands.
 */
export function describeSceneShapeElement(element: SceneShapeElement): string {
  switch (element.kind) {
    case ESceneShapeElement.SPHERE:
      return `Sphere, ${formatSceneMetres(element.radius)} radius at ${formatScenePoint(element.center)}`;

    case ESceneShapeElement.BOX: {
      const sides: Array<string> = [element.xAxis, element.yAxis, element.zAxis].map((axis: Vector3d) => {
        const { x, y, z }: IPoint3 = toPoint3(axis);

        return formatSceneMetres(Math.hypot(x, y, z));
      });

      return `Box, ${sides.join(" × ")} at ${formatScenePoint(element.translation)}`;
    }

    default:
      return assertExhaustive(element);
  }
}

/**
 * @param shape - The volume a shape encloses.
 * @returns Its spheres and boxes, one a line.
 */
export function describeSceneShape(shape: SceneShape): string {
  return formatSceneList(shape.elements, describeSceneShapeElement);
}
