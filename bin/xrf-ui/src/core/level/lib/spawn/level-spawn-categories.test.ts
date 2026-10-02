import { describe, expect, it } from "@jest/globals";

import { ELevelSpawnCategory } from "@/core/ipc/types/xrf-app";
import { DEFAULT_LEVEL_VIEW_OPTIONS } from "@/core/level/lib/view/level-view-options";

import { ILevelSpawnCategoryEntry, LEVEL_SPAWN_CATEGORIES, toLevelSpawnVisibility } from "./level-spawn-categories";

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

  it("shows the categories the options switch on", () => {
    expect(
      toLevelSpawnVisibility({ ...DEFAULT_LEVEL_VIEW_OPTIONS, isSpawnedItems: false, isSpawnedLamps: false })
    ).toEqual(new Set([ELevelSpawnCategory.PROPS, ELevelSpawnCategory.WEAPONS]));
  });
});
