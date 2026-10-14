import { IPoint3, toXraySpace } from "@xrf/math";
import { Maybe, Nullable } from "@xrf/types";

import { LevelSpawnObject, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { ERenderLevelHit, RenderLevelHit } from "@/core/ipc/types/xrf-renderer";

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
