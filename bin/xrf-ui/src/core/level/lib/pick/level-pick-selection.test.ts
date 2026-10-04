import { describe, expect, it } from "@jest/globals";

import { ERenderSelectionTarget } from "@/core/ipc/types/xrf-renderer";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { toLevelPickSelection } from "@/core/level/lib/pick/level-pick-selection";
import { mockLevelSpawnObject } from "@/fixtures/mocks/level.mocks";

const POINT = { x: 0, y: 0, z: 0 };

describe("toLevelPickSelection", () => {
  it("selects a picked spawned object, a picked surface, and nothing for none", () => {
    const spawn = toLevelPickSelection({
      kind: ELevelPick.SPAWN,
      object: mockLevelSpawnObject({ index: 4 }),
      point: POINT,
      visual: "crate",
    });
    const surface = toLevelPickSelection({
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: 2,
      place: 9,
      point: POINT,
      sector: 3,
      shaderId: 17,
    });

    expect(spawn?.target).toEqual({ kind: ERenderSelectionTarget.SPAWN, object: 4 });
    expect(surface?.target).toEqual({
      kind: ERenderSelectionTarget.SURFACE,
      mesh: 2,
      place: 9,
      sector: 3,
      shaderId: 17,
    });
    expect(toLevelPickSelection(null)).toBeNull();
  });

  // An impostor stands in for a clump of trees: it is no surface of the sector's to outline.
  it("selects nothing of a clump drawn as its impostor", () => {
    expect(
      toLevelPickSelection({
        isImpostor: true,
        kind: ELevelPick.SURFACE,
        mesh: null,
        place: 0,
        point: POINT,
        sector: 3,
        shaderId: 17,
      })
    ).toBeNull();
  });
});
