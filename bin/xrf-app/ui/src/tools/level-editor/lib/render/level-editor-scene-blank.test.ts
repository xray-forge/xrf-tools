import { describe, expect, it } from "@jest/globals";
import { ESceneToolKind, SceneDrawDescription } from "@xrf/ipc/types/xrf-app-levels";

import {
  mockSceneDrawDescription,
  mockSceneObjectsDescription,
  mockSceneObjectTool,
  mockSceneSpawn,
} from "@/tools/level-editor/fixtures/scene.mocks";
import {
  ELevelEditorSceneBlank,
  toLevelEditorSceneBlank,
} from "@/tools/level-editor/lib/render/level-editor-scene-blank";

describe("toLevelEditorSceneBlank", () => {
  it("says nothing for a scene that draws an object", () => {
    expect(toLevelEditorSceneBlank(mockSceneObjectsDescription(), mockSceneDrawDescription())).toBeNull();
  });

  it("says nothing until both the objects and the drawing are read", () => {
    const blank: SceneDrawDescription = { ...mockSceneDrawDescription(), bounds: null };

    expect(toLevelEditorSceneBlank(null, blank)).toBeNull();
    expect(toLevelEditorSceneBlank(mockSceneObjectsDescription(), null)).toBeNull();
  });

  // A scene decompiled before its geometry holds helpers alone.
  it("tells a scene holding no scene object", () => {
    expect(
      toLevelEditorSceneBlank(
        { problems: [], tools: [mockSceneObjectTool(ESceneToolKind.SPAWN, [mockSceneSpawn(0, "actor", "actor")])] },
        { ...mockSceneDrawDescription(), bounds: null }
      )
    ).toBe(ELevelEditorSceneBlank.NO_SCENE_OBJECTS);
  });

  it("tells a scene whose library objects were none of them drawn", () => {
    expect(
      toLevelEditorSceneBlank(mockSceneObjectsDescription(), { ...mockSceneDrawDescription(), bounds: null })
    ).toBe(ELevelEditorSceneBlank.NOTHING_FOUND);
  });
});
