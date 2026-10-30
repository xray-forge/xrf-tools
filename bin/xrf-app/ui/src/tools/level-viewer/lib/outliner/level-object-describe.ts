import {
  ELevelObjectId,
  ELevelObjectShape,
  ELevelRestriction,
  LevelObject,
  LevelObjectPatrol,
  LevelObjectShape,
  LevelPatrolLink,
  LevelRestriction,
} from "@xrf/ipc/types/xrf-app-levels";
import { Vector3d } from "@xrf/ipc/types/xrf-math";
import { toPoint3 } from "@xrf/math";
import { assertExhaustive, Maybe } from "@xrf/types";

import { toLevelObjectKey } from "@/core/level/lib/objects/level-object-kinds";

/** Metres as the Inspector writes them: two decimals, no trailing zeros. */
function formatMetres(value: number): string {
  return `${Number(value.toFixed(2))} m`;
}

/**
 * @param restriction - Whom a logic volume keeps where.
 * @returns It as the Inspector says it.
 */
export function describeLevelRestriction(restriction: LevelRestriction): string {
  switch (restriction) {
    case ELevelRestriction.NONE:
      return "No one by itself";
    case ELevelRestriction.OUT:
      return "Keeps out";
    case ELevelRestriction.IN:
      return "Keeps in";
    default:
      return assertExhaustive(restriction);
  }
}

/**
 * @param point - A patrol point.
 * @param objects - The level's objects, its path and the path's other points among them.
 * @returns Each point its path leads on to from it, by name, with how likely that way is taken; none for an object
 *   that is no patrol point.
 */
export function listLevelPatrolLeads(point: LevelObject, objects: ReadonlyArray<LevelObject>): Array<string> {
  if (point.id.source !== ELevelObjectId.PATROL_POINT || !point.parent) {
    return [];
  }

  const { path, point: from } = point.id;
  const parentKey: string = toLevelObjectKey(point.parent);
  const patrol: Maybe<LevelObjectPatrol> = objects.find(
    (it: LevelObject) => toLevelObjectKey(it.id) === parentKey
  )?.patrol;
  const names: ReadonlyMap<number, string> = new Map(
    objects.flatMap((it: LevelObject): Array<[number, string]> =>
      it.id.source === ELevelObjectId.PATROL_POINT && it.id.path === path ? [[it.id.point, it.name]] : []
    )
  );

  return (patrol?.links ?? [])
    .filter((link: LevelPatrolLink) => link.from === from)
    .map(
      (link: LevelPatrolLink) =>
        `${names.get(link.to) ?? `point ${link.to}`} (${Number((link.weight ?? 0).toFixed(2))})`
    );
}

/**
 * @param shape - One shape of a logic volume, in its own space.
 * @returns Its kind and size: a sphere's radius, a box's three sides.
 */
export function describeLevelShape(shape: LevelObjectShape): string {
  if (shape.kind === ELevelObjectShape.SPHERE) {
    return `Sphere, ${formatMetres(shape.radius ?? 0)} radius`;
  }

  const sides: Array<string> = shape.axes.map((axis: Vector3d) => {
    const { x, y, z } = toPoint3(axis);

    return formatMetres(Math.hypot(x, y, z));
  });

  return `Box, ${sides.join(" × ")}`;
}
