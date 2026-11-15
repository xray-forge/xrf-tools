import { describe, expect, it } from "@jest/globals";
import { ELevelObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderSelectionTarget } from "@xrf/ipc/types/xrf-renderer";

import { mockLevelObject, mockLevelVolumeObject } from "@/core/level/fixtures/level.mocks";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { toLevelPickSelection, toLevelSpawnSelection } from "@/core/level/lib/pick/level-pick-selection";

const POINT = { x: 0, y: 0, z: 0 };

describe("toLevelSpawnSelection", () => {
  it("outlines a model by its index in the colour a level's picks are outlined in", () => {
    const picked = toLevelPickSelection({
      kind: ELevelPick.OBJECT,
      object: mockLevelObject({ id: { index: 7, source: ELevelObjectId.SPAWN } }),
      point: POINT,
      visual: "crate",
    });

    expect(toLevelSpawnSelection(7)).toEqual(picked);
  });
});

describe("toLevelPickSelection", () => {
  it("selects a picked spawned model, a picked surface, and nothing for none", () => {
    const spawn = toLevelPickSelection({
      kind: ELevelPick.OBJECT,
      object: mockLevelObject({ id: { index: 4, source: ELevelObjectId.SPAWN } }),
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

  // A volume draws no model, so there is nothing of it in the frame to outline.
  it("selects nothing of an object drawn as nothing", () => {
    expect(
      toLevelPickSelection({ kind: ELevelPick.OBJECT, object: mockLevelVolumeObject(2), point: POINT, visual: "" })
    ).toBeNull();
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
