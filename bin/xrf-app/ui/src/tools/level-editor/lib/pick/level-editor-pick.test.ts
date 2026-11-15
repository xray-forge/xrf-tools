import { describe, expect, it } from "@jest/globals";
import { ESceneToolKind, SceneObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderLevelHit, ERenderSelectionTarget } from "@xrf/ipc/types/xrf-renderer";

import { toLevelSpawnSelection } from "@/core/level/lib/pick/level-pick-selection";
import { mockSceneObjectsDescription } from "@/tools/level-editor/fixtures/scene.mocks";
import { toSceneHelperId } from "@/tools/level-editor/lib/helpers/scene-helper-id";
import {
  isSceneObjectListed,
  toLevelEditorPick,
  toLevelEditorSelection,
} from "@/tools/level-editor/lib/pick/level-editor-pick";

const BOX: SceneObjectId = { index: 1, tool: ESceneToolKind.SCENE_OBJECT };
const STALKER: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };

describe("toLevelEditorPick", () => {
  it("names a model by its place among the scene objects and a helper by the object its id carries", () => {
    const objects = mockSceneObjectsDescription();

    expect(toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] }, objects)).toEqual(
      BOX
    );
    expect(
      toLevelEditorPick(
        { depth: 1, kind: ERenderLevelHit.HELPER, object: toSceneHelperId(STALKER) ?? -1, point: [0, 0, 0] },
        objects
      )
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
      toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.SPAWN, object: 9, point: [0, 0, 0] }, objects)
    ).toBeNull();
    expect(toLevelEditorPick({ depth: 1, kind: ERenderLevelHit.SPAWN, object: 0, point: [0, 0, 0] }, null)).toBeNull();
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
  it("outlines a scene object's model, and nothing for a helped object or none", () => {
    expect(toLevelEditorSelection(BOX)).toEqual(toLevelSpawnSelection(1));
    expect(toLevelEditorSelection(BOX)?.target).toEqual({ kind: ERenderSelectionTarget.SPAWN, object: 1 });
    expect(toLevelEditorSelection(STALKER)).toBeNull();
    expect(toLevelEditorSelection(null)).toBeNull();
  });
});
