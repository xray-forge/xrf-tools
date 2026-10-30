import { describe, expect, it } from "@jest/globals";
import { ELevelObjectKind, LEVEL_OBJECT_KINDS, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";

import { getLevelVisibleOption, ILevelVisibleKindEntry, LEVEL_VISIBLE_KINDS } from "./level-object-visibility";

describe("LEVEL_VISIBLE_KINDS", () => {
  // A kind the backend draws but the viewer cannot switch would be drawn whatever the view asks.
  it("switches every kind the kind table draws, once, each by an option of its own", () => {
    const drawn: Array<string> = LEVEL_OBJECT_KINDS.filter((entry: LevelObjectKindEntry) => entry.isDrawn).map(
      (entry: LevelObjectKindEntry) => entry.kind
    );

    expect(LEVEL_VISIBLE_KINDS.map((entry: ILevelVisibleKindEntry) => entry.kind)).toEqual(drawn);
    expect(new Set(LEVEL_VISIBLE_KINDS.map((entry: ILevelVisibleKindEntry) => entry.option)).size).toBe(drawn.length);
  });
});

describe("getLevelVisibleOption", () => {
  it("names no option for a kind with no picture of its own", () => {
    expect(getLevelVisibleOption(ELevelObjectKind.WEAPONS)).toBe("isSpawnedWeapons");
    expect(getLevelVisibleOption(ELevelObjectKind.RESTRICTORS)).toBeNull();
  });
});
