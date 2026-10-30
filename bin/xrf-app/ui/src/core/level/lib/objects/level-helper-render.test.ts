import { describe, expect, it } from "@jest/globals";
import { ELevelObjectKind, LEVEL_OBJECT_KINDS, LevelObjectKindEntry } from "@xrf/ipc/types/xrf-app-levels";

import { mockLevelObject, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import { DEFAULT_LEVEL_HELPER_OPTIONS } from "@/core/level/lib/objects/level-helper-options";
import { toLevelHelperRenderOptions } from "@/core/level/lib/objects/level-helper-render";

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
      []
    );

    expect(options.isShown).toBe(true);
    expect(options.kinds & toBit(ELevelObjectKind.PROPS)).not.toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.ANOMALIES)).not.toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.RESTRICTORS)).toBe(0);
    expect(options.kinds & toBit(ELevelObjectKind.STALKERS)).toBe(0);
  });

  it("names the selected and hidden objects by their spawn indices, ascending", () => {
    const options = toLevelHelperRenderOptions(
      { ...DEFAULT_LEVEL_HELPER_OPTIONS, isShown: true },
      [mockLevelVolumeObject(9), mockLevelVolumeObject(2)],
      [mockLevelObject()]
    );

    expect(options.selected).toEqual([2, 9]);
    expect(options.hidden).toEqual([0]);
  });
});
