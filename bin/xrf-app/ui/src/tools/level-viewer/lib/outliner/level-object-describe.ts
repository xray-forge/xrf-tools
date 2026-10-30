import {
  ELevelObjectShape,
  ELevelRestriction,
  LevelObjectShape,
  LevelRestriction,
} from "@xrf/ipc/types/xrf-app-levels";
import { Vector3d } from "@xrf/ipc/types/xrf-math";
import { toPoint3 } from "@xrf/math";
import { assertExhaustive } from "@xrf/types";

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
