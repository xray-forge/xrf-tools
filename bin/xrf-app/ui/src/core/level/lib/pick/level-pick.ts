import { LevelObject, LevelObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { ERenderLevelHit, RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";
import { IPoint3, toXraySpace } from "@xrf/math";
import { Maybe, Nullable } from "@xrf/types";

import { getLevelSpawnIndex, toLevelObjectKey } from "@/core/level/lib/objects/level-object-kinds";
import { toLevelObjectPosition } from "@/core/level/lib/objects/level-object-sphere";
import { findLevelSurface } from "@/core/level/lib/surface/level-surface-summary";

/** What a click in the viewport picked. */
export enum ELevelPick {
  /** A surface the level compiled, which a sector draws by a shader table entry. */
  SURFACE = "surface",
  /** An object placed in the level. */
  OBJECT = "object",
}

/**
 * Something of the open level a click in the viewport picked, and where the click's ray met it, in the level's own
 * coordinates.
 */
export type TLevelPick =
  | {
      kind: ELevelPick.SURFACE;
      sector: number;
      /** The shader table entry drawing it. */
      shaderId: number;
      /** The sector's instanced mesh it is one place of, or null for its baked geometry and its impostors. */
      mesh: Nullable<number>;
      /** Which place of the mesh or impostor run it is, or null for the baked geometry. */
      place: Nullable<number>;
      /** Whether it is a clump of trees drawn as its impostor. */
      isImpostor: boolean;
      point: IPoint3;
    }
  | {
      kind: ELevelPick.OBJECT;
      object: LevelObject;
      /** The visual it is drawn as, as its spawn names it; empty for one drawn as nothing. */
      visual: string;
      point: IPoint3;
    };

/** A pick of a level object. */
export type TLevelObjectPick = Extract<TLevelPick, { kind: ELevelPick.OBJECT }>;

/**
 * @param object - A level object.
 * @param objects - The level's objects it is one of, which name its visual.
 * @param point - Where it was picked, in the level's own coordinates; where it stands by default.
 * @returns It as picked.
 */
export function toLevelObjectPick(
  object: LevelObject,
  objects: Nullable<LevelObjectsDescription>,
  point: IPoint3 = toLevelObjectPosition(object.transform)
): TLevelObjectPick {
  const visual: Nullable<number> = object.spawn?.visual ?? null;

  return {
    kind: ELevelPick.OBJECT,
    object,
    point,
    visual: visual === null ? "" : (objects?.visuals[visual] ?? ""),
  };
}

/**
 * @param objects - The level's objects.
 * @param index - An object, by its index among the spawn's.
 * @returns The object the spawn places at that index, or null for none listed.
 */
export function findLevelSpawnObject(objects: Nullable<LevelObjectsDescription>, index: number): Nullable<LevelObject> {
  return objects?.objects.find((it: LevelObject) => getLevelSpawnIndex(it.id) === index) ?? null;
}

/**
 * @param hit - What a native viewport named under a point.
 * @param objects - The level's objects as held, which a picked object is found among.
 * @returns It as the level names what is picked, in the level's own coordinates; null for an object not held.
 */
export function toLevelPick(hit: RenderLevelHit, objects: Nullable<LevelObjectsDescription>): Nullable<TLevelPick> {
  const [x, y, z] = hit.point;
  const point: IPoint3 = toXraySpace({ x: x ?? 0, y: y ?? 0, z: z ?? 0 });

  if (hit.kind === ERenderLevelHit.SPAWN || hit.kind === ERenderLevelHit.HELPER) {
    // A drawn model carries its spawn index, a helper its object's place in the listing.
    const object: Nullable<LevelObject> =
      hit.kind === ERenderLevelHit.SPAWN
        ? findLevelSpawnObject(objects, hit.object)
        : (objects?.objects[hit.object] ?? null);

    return object ? toLevelObjectPick(object, objects, point) : null;
  }

  return {
    isImpostor: hit.isImpostor,
    kind: ELevelPick.SURFACE,
    mesh: hit.mesh,
    place: hit.place,
    point,
    sector: hit.sector,
    shaderId: hit.shaderId,
  };
}

/** The parts of an open level a pick is named among, as held. */
export interface ILevelPickParts {
  objects: Nullable<LevelObjectsDescription>;
  /** The shader table, indexed by the shader id a surface carries. */
  surfaces: Nullable<ReadonlyArray<XraySurfaceDescriptor>>;
}

/**
 * @param object - A level object as it was listed.
 * @param before - The level's objects as listed then, which name its parent.
 * @param after - The level's objects as listed now.
 * @returns It as listed now, found by its kind and name, the first that matches; a part, a patrol point among them,
 *   under its parent found the same way, since its name repeats across parents. Null where it is gone.
 */
export function relocateLevelObject(
  object: LevelObject,
  before: Nullable<LevelObjectsDescription>,
  after: Nullable<LevelObjectsDescription>
): Nullable<LevelObject> {
  function isNamed(it: LevelObject): boolean {
    return it.kind === object.kind && it.name === object.name;
  }

  if (!object.parent) {
    return after?.objects.find((it: LevelObject) => !it.parent && isNamed(it)) ?? null;
  }

  const parentKey: string = toLevelObjectKey(object.parent);
  const parent: Maybe<LevelObject> = before?.objects.find((it: LevelObject) => toLevelObjectKey(it.id) === parentKey);
  const moved: Nullable<LevelObject> = parent ? relocateLevelObject(parent, before, after) : null;

  if (!moved) {
    return null;
  }

  const movedKey: string = toLevelObjectKey(moved.id);

  return (
    after?.objects.find((it: LevelObject) => it.parent && toLevelObjectKey(it.parent) === movedKey && isNamed(it)) ??
    null
  );
}

/**
 * @param objects - Level objects as they were listed.
 * @param before - The level's objects as listed then.
 * @param after - The level's objects as listed now.
 * @returns Each as listed now, in the same order, those gone left out.
 */
export function relocateLevelObjects(
  objects: ReadonlyArray<LevelObject>,
  before: Nullable<LevelObjectsDescription>,
  after: Nullable<LevelObjectsDescription>
): Array<LevelObject> {
  return objects
    .map((object: LevelObject) => relocateLevelObject(object, before, after))
    .filter((object: Nullable<LevelObject>): object is LevelObject => object !== null);
}

/**
 * Finds what was picked again once the level's parts are read anew, which may number it afresh: a level object by its
 * kind and name, a surface by the shader and textures of the entry drawing it, the first that matches either way. A
 * surface keeps the place it was picked at.
 *
 * @param pick - What was picked.
 * @param before - The parts it was picked among.
 * @param after - The parts as read now.
 * @returns It as the parts read now name it, the same pick where they name it alike; null where it is gone.
 */
export function relocateLevelPick(
  pick: TLevelPick,
  before: ILevelPickParts,
  after: ILevelPickParts
): Nullable<TLevelPick> {
  if (pick.kind === ELevelPick.OBJECT) {
    if (after.objects === before.objects) {
      return pick;
    }

    const object: Nullable<LevelObject> = relocateLevelObject(pick.object, before.objects, after.objects);

    return object ? toLevelObjectPick(object, after.objects) : null;
  }

  // A table first read now is the one the surface was picked from.
  if (after.surfaces === before.surfaces || !before.surfaces) {
    return pick;
  }

  const entry: Maybe<XraySurfaceDescriptor> = before.surfaces[pick.shaderId];
  const shaderId: Nullable<number> = entry && after.surfaces ? findLevelSurface(entry, after.surfaces) : null;

  if (shaderId === null) {
    return null;
  }

  return shaderId === pick.shaderId ? pick : { ...pick, shaderId };
}
