import { describe, expect, it } from "@jest/globals";
import {
  ELevelObjectKind,
  LEVEL_OBJECT_KINDS,
  LEVEL_SOUND_EMITTERS_HELPER_KIND,
  LevelObject,
  LevelObjectKindEntry,
} from "@xrf/ipc/types/xrf-app-levels";

import { mockLevelObject, mockLevelPatrol, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import { DEFAULT_LEVEL_HELPER_OPTIONS } from "@/core/level/lib/objects/level-helper-options";
import {
  toLevelHelperRenderOptions,
  toLevelListingPlaceMap,
  toLevelSpawnIndices,
} from "@/core/level/lib/objects/level-helper-render";

/** A kind's bit, as the backend numbers kinds by their place in the kind table. */
function toBit(kind: ELevelObjectKind): number {
  return 1 << LEVEL_OBJECT_KINDS.findIndex((entry: LevelObjectKindEntry) => entry.kind === kind);
}

describe("toLevelHelperRenderOptions", () => {
  it("switches each kind by its place in the kind table, those never switched as the table says", () => {
    const options = toLevelHelperRenderOptions(
      {
        ...DEFAULT_LEVEL_HELPER_OPTIONS,
        isShown: true,
        kinds: { [ELevelObjectKind.RESTRICTORS]: false, [ELevelObjectKind.PROPS]: true },
      },
      { hidden: [], hovered: null, selected: [] },
      new Map()
    );

    expect(options.isShown).toBe(true);
    expect(options.kinds & toBit(ELevelObjectKind.PROPS)).not.toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.ANOMALIES)).not.toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.RESTRICTORS)).toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.STALKERS)).toBe(0);
    expect(options.kinds & (1 << LEVEL_SOUND_EMITTERS_HELPER_KIND)).toBe(0);
  });

  it("switches the sound emitters by the bit past the kind table's", () => {
    const options = toLevelHelperRenderOptions(
      { ...DEFAULT_LEVEL_HELPER_OPTIONS, isEmitterShown: true, isShown: true },
      { hidden: [], hovered: null, selected: [] },
      new Map()
    );

    expect(LEVEL_SOUND_EMITTERS_HELPER_KIND).toBe(LEVEL_OBJECT_KINDS.length);
    expect(options.kinds & (1 << LEVEL_SOUND_EMITTERS_HELPER_KIND)).not.toBe(0);
  });

  // The backend names each helper by its object's place in the listing, whatever source the object comes from.
  it("names the selected, hidden and hovered objects by their places in the listing, ascending", () => {
    const [path, first, second] = mockLevelPatrol(0, "walk", 2) as [LevelObject, LevelObject, LevelObject];
    const places: ReadonlyMap<string, number> = toLevelListingPlaceMap([
      mockLevelObject(),
      mockLevelVolumeObject(9),
      path,
      first,
      second,
    ]);
    const options = toLevelHelperRenderOptions(
      { ...DEFAULT_LEVEL_HELPER_OPTIONS, isRestFaded: true, isShown: true },
      {
        hidden: [mockLevelObject(), mockLevelVolumeObject(7)],
        hovered: path,
        selected: [second, mockLevelVolumeObject(9)],
      },
      places
    );

    expect(options.selected).toEqual([1, 4]);
    expect(options.hidden).toEqual([0]);
    expect(options.hovered).toBe(2);
    expect(options.isRestFaded).toBe(true);
    expect(
      toLevelHelperRenderOptions(
        DEFAULT_LEVEL_HELPER_OPTIONS,
        { hidden: [], hovered: mockLevelVolumeObject(7), selected: [] },
        places
      ).hovered
    ).toBeNull();
  });
});

describe("toLevelSpawnIndices", () => {
  it("names only what the spawn places, by its index there, ascending", () => {
    expect(
      toLevelSpawnIndices([mockLevelVolumeObject(9), ...mockLevelPatrol(0, "walk", 1), mockLevelObject()])
    ).toEqual([0, 9]);
  });
});
