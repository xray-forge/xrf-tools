import { LevelSpawnObject, LevelSpawnObjectsDescription } from "@xrf/ipc/types/xrf-app-levels";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import { ERenderLevelHit, RenderLevelHit } from "@xrf/ipc/types/xrf-renderer";
import { IPoint3, toXraySpace } from "@xrf/math";
import { Maybe, Nullable } from "@xrf/types";

import { toLevelSpawnPosition } from "@/core/level/lib/spawn/level-spawn-sphere";
import { findLevelSurface } from "@/core/level/lib/surface/level-surface-summary";

/** What a click in the viewport picked. */
export enum ELevelPick {
  /** A surface the level compiled, which a sector draws by a shader table entry. */
  SURFACE = "surface",
  /** An object the level's spawn places. */
  SPAWN = "spawn",
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
      kind: ELevelPick.SPAWN;
      object: LevelSpawnObject;
      /** The visual it stands as, as its spawn names it. */
      visual: string;
      point: IPoint3;
    };

/**
 * @param hit - What a native viewport named under a point.
 * @param spawn - The level's spawned objects as held, which a picked object is found among.
 * @returns It as the level names what is picked, in the level's own coordinates; null for an object not held.
 */
export function toLevelPick(hit: RenderLevelHit, spawn: Nullable<LevelSpawnObjectsDescription>): Nullable<TLevelPick> {
  const [x, y, z] = hit.point;
  const point: IPoint3 = toXraySpace({ x: x ?? 0, y: y ?? 0, z: z ?? 0 });

  if (hit.kind === ERenderLevelHit.SPAWN) {
    const object: Maybe<LevelSpawnObject> = spawn?.objects.find((it: LevelSpawnObject) => it.index === hit.object);

    return object ? { kind: ELevelPick.SPAWN, object, point, visual: spawn?.visuals[object.visual] ?? "" } : null;
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
  spawn: Nullable<LevelSpawnObjectsDescription>;
  /** The shader table, indexed by the shader id a surface carries. */
  surfaces: Nullable<ReadonlyArray<XraySurfaceDescriptor>>;
}

/**
 * Finds what was picked again once the level's parts are read anew, which may number it afresh: a spawned object by its
 * name, a surface by the shader and textures of the entry drawing it, the first that matches either way. A surface
 * keeps the place it was picked at.
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
  if (pick.kind === ELevelPick.SPAWN) {
    if (after.spawn === before.spawn) {
      return pick;
    }

    const object: Maybe<LevelSpawnObject> = after.spawn?.objects.find(
      (it: LevelSpawnObject) => it.name === pick.object.name
    );

    return object
      ? {
          kind: ELevelPick.SPAWN,
          object,
          point: toLevelSpawnPosition(object.transform),
          visual: after.spawn?.visuals[object.visual] ?? "",
        }
      : null;
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
