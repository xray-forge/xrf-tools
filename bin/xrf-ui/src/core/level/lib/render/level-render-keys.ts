import { Nullable } from "@xrf/types";

import { ELevelSpawnCategory, LevelSpawnCategory } from "@/core/ipc/types/xrf-app";

/** The keys a level is put to the renderer under. */
export const LEVEL_RENDER_KEYS = {
  axes: "axes",
  extent: "extent",
  extentBox: "extent-box",
  grid: "grid",
  /** One run of a sector's impostors drawn with one surface. */
  impostorGroup: (sector: number, group: number): string => `sector:${sector}:impostors:${group}`,
  /** The quad every impostor draws over, its corners numbered in its coordinate. */
  impostorQuad: "impostor-quad",
  /** A sector's impostor set, which its trees and its impostor draws both name. */
  impostors: (sector: number): string => `sector:${sector}:impostors`,
  /** One mesh a sector stands in many places. */
  instance: (sector: number, index: number): string => `sector:${sector}:instance:${index}`,
  /** Everything a sector bakes in place, one geometry with a group per surface. */
  sector: (sector: number): string => `sector:${sector}`,
  /** A visual the level's spawned objects stand as: its submeshes joined in one geometry, a group each. */
  spawnGeometry: (visual: number): string => `spawn:${visual}`,
  /** The objects of one category standing as a visual. */
  spawnObject: (visual: number, category: LevelSpawnCategory): string => `spawn:${visual}:${category}`,
  /** One submesh's surface of a visual, which every category's objects of it draw with. */
  spawnSurface: (visual: number, submesh: number): string => `spawn:${visual}:surface:${submesh}`,
  sun: "sun",
  /** One shader table entry, which every sector drawing it shares. */
  surface: (shaderId: number): string => `surface:${shaderId}`,
} as const;

/** What the key of an object a level put names, read back from it. */
export type TLevelRenderObjectKey =
  | {
      kind: "sector";
      sector: number;
      /** Which of its instanced meshes, or null for its baked geometry or an impostor run. */
      mesh: Nullable<number>;
      /** Which of its impostor runs, or null for anything else. */
      impostors: Nullable<number>;
    }
  | { kind: "spawn"; visual: number; category: LevelSpawnCategory };

/** A sector's baked geometry, one of its instanced meshes, or one of its impostor runs. */
const SECTOR_OBJECT_KEY: RegExp = /^sector:(\d+)(?::(instance|impostors):(\d+))?$/;

/** The objects of one category standing as one visual. */
const SPAWN_OBJECT_KEY: RegExp = /^spawn:(\d+):([a-z_]+)$/;

/** Every category a spawn object key may name. */
const SPAWN_CATEGORIES: ReadonlySet<string> = new Set(Object.values(ELevelSpawnCategory));

/** One shader table entry. */
const SURFACE_KEY: RegExp = /^surface:(\d+)$/;

/**
 * @param key - The key of an object the renderer drew.
 * @returns What of the level it names, or null for a key the level puts no object under.
 */
export function readLevelObjectKey(key: string): Nullable<TLevelRenderObjectKey> {
  const sector: Nullable<RegExpExecArray> = SECTOR_OBJECT_KEY.exec(key);

  if (sector) {
    const run: Nullable<number> = sector[3] === undefined ? null : Number(sector[3]);

    return {
      impostors: sector[2] === "impostors" ? run : null,
      kind: "sector",
      mesh: sector[2] === "instance" ? run : null,
      sector: Number(sector[1]),
    };
  }

  const spawn: Nullable<RegExpExecArray> = SPAWN_OBJECT_KEY.exec(key);

  return spawn && SPAWN_CATEGORIES.has(spawn[2])
    ? { category: spawn[2] as LevelSpawnCategory, kind: "spawn", visual: Number(spawn[1]) }
    : null;
}

/**
 * @param key - The key of a surface the renderer drew with.
 * @returns The shader table entry it is, or null for a surface of anything else.
 */
export function readLevelSurfaceKey(key: string): Nullable<number> {
  const surface: Nullable<RegExpExecArray> = SURFACE_KEY.exec(key);

  return surface ? Number(surface[1]) : null;
}
