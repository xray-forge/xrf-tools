import { describe, expect, it } from "@jest/globals";
import { ESceneToolKind, SceneObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderLevelHit, ERenderSelectionTarget } from "@xrf/ipc/types/xrf-renderer";

import { toLevelSpawnSelection } from "@/core/level/lib/pick/level-pick-selection";
import { mockSceneObjectsDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import {
  isSceneObjectListed,
  toLevelEditorPick,
  toLevelEditorSelection,
} from "@/tools/level-editor/lib/pick/level-editor-pick";
import { toSceneRenderId } from "@/tools/level-editor/lib/render/scene-render-id";

const BOX: SceneObjectId = { index: 1, tool: ESceneToolKind.SCENE_OBJECT };
const STALKER: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };

/** The render id `object` carries. */
function toId(object: SceneObjectId): number {
  return toSceneRenderId(object) ?? -1;
}

describe("toLevelEditorPick", () => {
  it("names a model and a helper alike by the object their render id carries", () => {
    const objects = mockSceneObjectsDescription();

    expect(
      toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.SPAWN, object: toId(BOX), point: [0, 0, 0] }, objects)
    ).toEqual(BOX);
    expect(
      toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.HELPER, object: toId(STALKER), point: [0, 0, 0] }, objects)
    ).toEqual(STALKER);
    // A spawn point drawn as its model answers its spawn point, as its helper does.
    expect(
      toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.SPAWN, object: toId(STALKER), point: [0, 0, 0] }, objects)
    ).toEqual(STALKER);
  });

  it("picks nothing for nothing hit, a surface, or an object the listing does not hold", () => {
    const objects = mockSceneObjectsDescription();

    expect(toLevelEditorPick(null, objects)).toBeNull();
    expect(
      toLevelEditorPick(
        {
          depth: 1,
          isImpostor: false,
          kind: ERenderLevelHit.SURFACE,
          mesh: null,
          place: null,
          point: [0, 0, 0],
          sector: 0,
          shaderId: 0,
        },
        objects
      )
    ).toBeNull();
    expect(
      toLevelEditorPick(
        {
          depth: 1,
          kind: ERenderLevelHit.SPAWN,
          object: toId({ index: 9, tool: ESceneToolKind.SCENE_OBJECT }),
          point: [0, 0, 0],
        },
        objects
      )
    ).toBeNull();
    // A bare index names the first tool's object, a group's, which the listing does not hold.
    expect(
      toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] }, objects)
    ).toBeNull();
    expect(
      toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.SPAWN, object: toId(BOX), point: [0, 0, 0] }, null)
    ).toBeNull();
  });
});

describe("isSceneObjectListed", () => {
  it("finds an object by its tool and index", () => {
    const objects = mockSceneObjectsDescription();

    expect(isSceneObjectListed(objects, STALKER)).toBe(true);
    expect(isSceneObjectListed(objects, { index: 2, tool: ESceneToolKind.SCENE_OBJECT })).toBe(false);
    expect(isSceneObjectListed(objects, { index: 0, tool: ESceneToolKind.LIGHT })).toBe(false);
  });
});

describe("toLevelEditorSelection", () => {
  it("outlines the model standing as the selected object by its render id, whatever its tool", () => {
    expect(toLevelEditorSelection(BOX)).toEqual(toLevelSpawnSelection(toId(BOX)));
    expect(toLevelEditorSelection(BOX)?.target).toEqual({ kind: ERenderSelectionTarget.SPAWN, object: 0x0200_0001 });
    expect(toLevelEditorSelection(STALKER)?.target).toEqual({
      kind: ERenderSelectionTarget.SPAWN,
      object: 0x0600_0002,
    });
  });

  it("outlines nothing for no selection", () => {
    expect(toLevelEditorSelection(null)).toBeNull();
  });
});
