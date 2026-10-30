import { describe, expect, it } from "@jest/globals";
import { ELevelObjectKind, LEVEL_OBJECT_KINDS, LevelObject, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";

import { mockLevelObject, mockLevelPatrol, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import { DEFAULT_LEVEL_HELPER_OPTIONS } from "@/core/level/lib/objects/level-helper-options";
import { toLevelHelperRenderOptions, toLevelSpawnIndices } from "@/core/level/lib/objects/level-helper-render";

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
      [],
      [],
      []
    );

    expect(options.isShown).toBe(true);
    expect(options.kinds & toBit(ELevelObjectKind.PROPS)).not.toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.ANOMALIES)).not.toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.RESTRICTORS)).toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.STALKERS)).toBe(0);
  });

  // The backend names each helper by its object's place in the listing, whatever source the object comes from.
  it("names the selected and hidden objects by their places in the listing, ascending", () => {
    const [path, first, second] = mockLevelPatrol(0, "walk", 2) as [LevelObject, LevelObject, LevelObject];
    const listed: Array<LevelObject> = [mockLevelObject(), mockLevelVolumeObject(9), path, first, second];
    const options = toLevelHelperRenderOptions(
      { ...DEFAULT_LEVEL_HELPER_OPTIONS, isShown: true },
      [second, mockLevelVolumeObject(9)],
      [mockLevelObject(), mockLevelVolumeObject(7)],
      listed
    );

    expect(options.selected).toEqual([1, 4]);
    expect(options.hidden).toEqual([0]);
  });
});

describe("toLevelSpawnIndices", () => {
  it("names only what the spawn places, by its index there, ascending", () => {
    expect(
      toLevelSpawnIndices([mockLevelVolumeObject(9), ...mockLevelPatrol(0, "walk", 1), mockLevelObject()])
    ).toEqual([0, 9]);
  });
});
