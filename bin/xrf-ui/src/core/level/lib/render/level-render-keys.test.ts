import { describe, expect, it } from "@jest/globals";

import { ELevelSpawnCategory } from "@/core/ipc/types/xrf-app";
import { LEVEL_RENDER_KEYS, readLevelObjectKey, readLevelSurfaceKey } from "@/core/level/lib/render/level-render-keys";

describe("level render keys", () => {
  it("reads back what each object key a level puts names", () => {
    expect(readLevelObjectKey(LEVEL_RENDER_KEYS.sector(12))).toEqual({
      impostors: null,
      kind: "sector",
      mesh: null,
      sector: 12,
    });
    expect(readLevelObjectKey(LEVEL_RENDER_KEYS.instance(12, 3))).toMatchObject({ impostors: null, mesh: 3 });
    expect(readLevelObjectKey(LEVEL_RENDER_KEYS.impostorGroup(12, 1))).toMatchObject({ impostors: 1, mesh: null });
    expect(readLevelObjectKey(LEVEL_RENDER_KEYS.spawnObject(7, ELevelSpawnCategory.WEAPONS))).toEqual({
      category: ELevelSpawnCategory.WEAPONS,
      kind: "spawn",
      visual: 7,
    });
  });

  it("reads nothing from a key no object is put under, and a shader entry only from an entry's surface", () => {
    expect(readLevelObjectKey(LEVEL_RENDER_KEYS.grid)).toBeNull();
    expect(readLevelObjectKey(LEVEL_RENDER_KEYS.impostors(12))).toBeNull();
    expect(readLevelObjectKey(LEVEL_RENDER_KEYS.spawnSurface(7, 0))).toBeNull();
    expect(readLevelSurfaceKey(LEVEL_RENDER_KEYS.surface(99))).toBe(99);
    expect(readLevelSurfaceKey(LEVEL_RENDER_KEYS.spawnSurface(7, 0))).toBeNull();
  });
});
