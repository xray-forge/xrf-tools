import { describe, expect, it } from "@jest/globals";
import {
  ESceneToolKind,
  SceneDrawDescription,
  SceneHelpersDescription,
  SceneHelperToolCount,
  SceneObjectDescription,
  SceneObjectsDescription,
} from "@xrf/ipc/types/xrf-app-levels";

import {
  MOCK_NO_SCENE_HELPERS,
  mockSceneDrawDescription,
  mockSceneHelpersDescription,
  mockSceneObject,
  mockSceneObjectsDescription,
  mockSceneObjectTool,
  mockSceneSpawn,
} from "@/tools/level-editor/fixtures/scene.mocks";
import {
  DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS,
  ILevelEditorHelperOptions,
} from "@/tools/level-editor/lib/helpers/level-editor-helper-options";
import {
  ELevelEditorSceneBlank,
  toLevelEditorSceneBlank,
} from "@/tools/level-editor/lib/render/level-editor-scene-blank";

/** What a scene's picture needs where it draws no object. */
const UNDRAWN: SceneDrawDescription = { ...mockSceneDrawDescription(), bounds: null };

/** Helpers as the editor draws them until told otherwise. */
const SHOWN: ILevelEditorHelperOptions = DEFAULT_LEVEL_EDITOR_HELPER_OPTIONS;

/** A scene decompiled before its geometry: spawn points and nothing drawn as a model. */
const SPAWNS_ALONE: SceneObjectsDescription = {
  problems: [],
  tools: [mockSceneObjectTool(ESceneToolKind.SPAWN, [mockSceneSpawn(0, "actor", "actor")])],
};

describe("toLevelEditorSceneBlank", () => {
  it("says nothing for a scene that draws an object", () => {
    expect(
      toLevelEditorSceneBlank(mockSceneObjectsDescription(), mockSceneDrawDescription(), MOCK_NO_SCENE_HELPERS, SHOWN)
    ).toBeNull();
  });

  it("says nothing until the objects, the drawing and the helpers are read", () => {
    expect(toLevelEditorSceneBlank(null, UNDRAWN, MOCK_NO_SCENE_HELPERS, SHOWN)).toBeNull();
    expect(toLevelEditorSceneBlank(mockSceneObjectsDescription(), null, MOCK_NO_SCENE_HELPERS, SHOWN)).toBeNull();
    expect(toLevelEditorSceneBlank(mockSceneObjectsDescription(), UNDRAWN, null, SHOWN)).toBeNull();
  });

  it("says nothing for a scene that draws helpers alone", () => {
    expect(toLevelEditorSceneBlank(SPAWNS_ALONE, UNDRAWN, mockSceneHelpersDescription(), SHOWN)).toBeNull();
  });

  it("tells a scene of helpers alone whose helpers are all switched off", () => {
    const helpers: SceneHelpersDescription = mockSceneHelpersDescription();

    expect(toLevelEditorSceneBlank(SPAWNS_ALONE, UNDRAWN, helpers, { ...SHOWN, isShown: false })).toBe(
      ELevelEditorSceneBlank.HELPERS_HIDDEN
    );
    expect(
      toLevelEditorSceneBlank(SPAWNS_ALONE, UNDRAWN, helpers, {
        isShown: true,
        tools: Object.fromEntries(helpers.tools.map((counted: SceneHelperToolCount) => [counted.tool, false])),
      })
    ).toBe(ELevelEditorSceneBlank.HELPERS_HIDDEN);
  });

  it("tells a scene holding no objects at all", () => {
    expect(toLevelEditorSceneBlank({ problems: [], tools: [] }, UNDRAWN, MOCK_NO_SCENE_HELPERS, SHOWN)).toBe(
      ELevelEditorSceneBlank.EMPTY
    );
  });

  it("tells a scene whose library objects were none of them drawn, where nothing else draws", () => {
    expect(toLevelEditorSceneBlank(mockSceneObjectsDescription(), UNDRAWN, MOCK_NO_SCENE_HELPERS, SHOWN)).toBe(
      ELevelEditorSceneBlank.NOTHING_FOUND
    );
  });

  it("tells a scene whose objects draw neither a model nor a helper", () => {
    const sector: SceneObjectDescription = mockSceneObject({
      id: { index: 0, tool: ESceneToolKind.SECTOR },
      name: "sector_0000",
    });

    expect(
      toLevelEditorSceneBlank(
        { problems: [], tools: [mockSceneObjectTool(ESceneToolKind.SECTOR, [sector])] },
        UNDRAWN,
        MOCK_NO_SCENE_HELPERS,
        SHOWN
      )
    ).toBe(ELevelEditorSceneBlank.NOTHING_DRAWN);
  });
});
