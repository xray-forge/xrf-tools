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
import { assertExhaustive, Maybe, Nullable } from "@xrf/types";

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

/** A count of milliseconds as the Inspector writes it: seconds, a decimal at most. */
function formatSeconds(milliseconds: number): string {
  return `${Number((milliseconds / 1000).toFixed(1))} s`;
}

/** A time of the game day, from its milliseconds past midnight, as a clock writes it. */
function formatDayTime(milliseconds: number): string {
  const minutes: number = Math.floor(milliseconds / 60_000) % (24 * 60);

  return `${String(Math.floor(minutes / 60)).padStart(2, "0")}:${String(minutes % 60).padStart(2, "0")}`;
}

/**
 * A static sound's window, from and to in milliseconds, as `SStaticSound` reads it: none, both nought, means the
 * sound's own default.
 *
 * @param window - From and to, in milliseconds.
 * @param kind - What the window measures: the game day's hours the sound plays in, how long each play lasts, or the
 *   pause between plays.
 * @returns It as the Inspector says it.
 */
export function describeLevelSoundWindow(
  window: ReadonlyArray<Nullable<number>>,
  kind: "active" | "play" | "pause"
): string {
  const [from = 0, to = 0] = window.map((it: Nullable<number>) => it ?? 0);

  if (from === 0 && to === 0) {
    return { active: "All day", pause: "Looped without a pause", play: "The whole sound" }[kind];
  }

  return kind === "active"
    ? `${formatDayTime(from)} to ${formatDayTime(to)}`
    : `${formatSeconds(from)} to ${formatSeconds(to)}`;
}

/**
 * @param color - An authored colour's channels, linear, from nought to one.
 * @returns Its channels as the Inspector writes them, two decimals each.
 */
export function describeLevelColor(color: ReadonlyArray<Nullable<number>>): string {
  return color.map((channel: Nullable<number>) => (channel ?? 0).toFixed(2)).join(" ");
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
