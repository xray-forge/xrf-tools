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
 * Entries of a list the Inspector writes out before counting the rest; a way or a sector can hold thousands.
 */
export const SCENE_LIST_LIMIT: number = 20;

/**
 * A point in the scene's space, one decimal an axis.
 */
export function formatScenePoint(vector: Vector3d): string {
  return formatLevelPoint(toPoint3(vector));
}

/**
 * Radian angles about `x`, `y` and `z` in degrees, one decimal each.
 */
export function formatSceneRotation(rotation: Vector3d): string {
  const { x, y, z }: IPoint3 = toPoint3(rotation);

  return `x ${formatDegrees(toDegrees(x), 1)} y ${formatDegrees(toDegrees(y), 1)} z ${formatDegrees(toDegrees(z), 1)}`;
}

/**
 * The editor camera's radian heading, pitch and bank in degrees, one decimal each.
 */
export function formatSceneFacing(rotation: Vector3d): string {
  const { x, y, z }: IPoint3 = toPoint3(rotation);

  return `h ${formatDegrees(toDegrees(x), 1)} p ${formatDegrees(toDegrees(y), 1)} b ${formatDegrees(toDegrees(z), 1)}`;
}

/**
 * A scale, two decimals an axis.
 */
export function formatSceneScale(scale: Vector3d): string {
  return `x ${formatNumber(scale.x, 2)} y ${formatNumber(scale.y, 2)} z ${formatNumber(scale.z, 2)}`;
}

/**
 * A length in metres to two decimals, or the absent mark for null or a non-finite value.
 */
export function formatSceneMetres(value: Nullable<number>): string {
  return value === null || !Number.isFinite(value) ? ABSENT_VALUE : `${formatNumber(value, 2)} m`;
}

/**
 * A colour's channels, space-separated, `digits` decimals each: zero for bytes.
 */
export function formatSceneColor(channels: ReadonlyArray<Nullable<number>>, digits: number = 2): string {
  return channels.map((channel: Nullable<number>) => formatNumber(channel, digits)).join(" ");
}

/**
 * A list one entry a line, entries past the limit counted rather than described; `None` for an empty list.
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
 * A light type's Inspector name; an unnamed type by its number.
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
 * A shape usage's Inspector name; an unnamed usage by its number.
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
 * A way type's Inspector name; an unnamed type by its number.
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
 * A shape's sphere or box: kind, size and position in its object's space.
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
 * A shape's spheres and boxes, one a line.
 */
export function describeSceneShape(shape: SceneShape): string {
  return formatSceneList(shape.elements, describeSceneShapeElement);
}
