import { describe, expect, it } from "@jest/globals";
import { ELevelSpawnCategory } from "@xrf/ipc/types/xrf-app-levels";

import { ILevelSpawnCategoryEntry, LEVEL_SPAWN_CATEGORIES } from "./level-spawn-categories";

describe("LEVEL_SPAWN_CATEGORIES", () => {
  // A category the backend classifies objects into but the viewer does not list is neither drawn nor switchable.
  it("lists every category the backend sorts spawned objects into, once, each by an option of its own", () => {
    const categories: Array<ELevelSpawnCategory> = LEVEL_SPAWN_CATEGORIES.map(
      (entry: ILevelSpawnCategoryEntry) => entry.category
    );

    expect([...categories].sort()).toEqual(Object.values(ELevelSpawnCategory).sort());
    expect(new Set(LEVEL_SPAWN_CATEGORIES.map((entry: ILevelSpawnCategoryEntry) => entry.option)).size).toBe(
      categories.length
    );
  });
});
