import { Nullable } from "@xrf/types";

import { LevelSpawnObject } from "@/core/ipc/types/xrf-app";
import { ILevelPoint } from "@/core/level/lib/camera/level-point";

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
      point: ILevelPoint;
    }
  | {
      kind: ELevelPick.SPAWN;
      object: LevelSpawnObject;
      /** The visual it stands as, as its spawn names it. */
      visual: string;
      point: ILevelPoint;
    };
