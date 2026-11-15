import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container, EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ESessionRootChange, SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { ESceneToolKind, SCENE_OBJECTS_PART, SceneObjectId, SceneOpenRequest } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import {
  mockSceneObjectsDescription,
  mockSceneObjectTool,
  mockScenePartResponses,
  mockSceneSpawn,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorSelectionService } from "./level-editor-selection.service";

const REQUEST: SceneOpenRequest = {
  engine: EXrayEngineChoice.AUTO,
  gameRoots: null,
  isDltx: false,
  objectsRoots: null,
  source: { path: "C:\\rawdata\\levels\\labx8.level" },
};

interface IEditor {
  container: Container;
  scene: LevelEditorSceneService;
  selection: LevelEditorSelectionService;
}

/** Builds the editor's two services and opens the fixture scene, each open a session of its own. */
async function openEditor(): Promise<IEditor> {
  const container: Container = mockContainer([LevelEditorSceneService, LevelEditorSelectionService]);
  const scene: LevelEditorSceneService = container.get(LevelEditorSceneService);
  let opened: number = 0;

  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: () => mockSessionSnapshot(mockSelectedSceneDescription(), `scene-${(opened += 1)}`),
    ...mockScenePartResponses(),
  });

  await scene.open(REQUEST);
  await flush();

  return { container, scene, selection: container.get(LevelEditorSelectionService) };
}

describe("LevelEditorSelectionService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("selects an object of the open scene by its tool and index, and nothing again", async () => {
    const { selection } = await openEditor();

    selection.select({ index: 2, tool: ESceneToolKind.SPAWN });

    expect(selection.selected?.name).toBe("stalker_2");
    expect(selection.selectedId).toEqual({ index: 2, tool: ESceneToolKind.SPAWN });

    selection.select(null);

    expect(selection.selected).toBeNull();
  });

  it("selects nothing for an object the scene does not list", async () => {
    const { selection } = await openEditor();

    selection.select({ index: 9, tool: ESceneToolKind.SPAWN });

    expect(selection.selected).toBeNull();
    expect(selection.selectedId).toBeNull();
  });

  // Ids are tool and index: the same id in another scene is another object.
  it("selects nothing once another scene opens, or the scene closes", async () => {
    const { scene, selection } = await openEditor();

    selection.select({ index: 0, tool: ESceneToolKind.SCENE_OBJECT });
    await scene.open(REQUEST);
    await flush();

    expect(scene.scene.sessionId).toBe("scene-2");
    expect(selection.selected).toBeNull();

    selection.select({ index: 0, tool: ESceneToolKind.SCENE_OBJECT });

    expect(selection.selected?.name).toBe("box_10");

    await scene.close();

    expect(selection.selected).toBeNull();
  });

  it("hides objects one by one, shows one again, and shows them all", async () => {
    const { selection } = await openEditor();
    const box: SceneObjectId = { index: 0, tool: ESceneToolKind.SCENE_OBJECT };
    const stalker: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };

    selection.toggleHidden(box);
    selection.toggleHidden(stalker);

    expect(selection.hidden).toEqual([box, stalker]);
    expect(selection.isHidden({ index: 2, tool: ESceneToolKind.SPAWN })).toBe(true);

    selection.toggleHidden(box);

    expect(selection.hidden).toEqual([stalker]);

    selection.revealAll();

    expect(selection.hidden).toEqual([]);
  });

  // What was hidden belongs to the scene's opening, not to the editor.
  it("hides nothing once another scene opens", async () => {
    const { scene, selection } = await openEditor();

    selection.toggleHidden({ index: 0, tool: ESceneToolKind.SCENE_OBJECT });
    await scene.open(REQUEST);
    await flush();

    expect(selection.hidden).toEqual([]);
  });

  it("notes the object under the pointer, keeping the one noted for the same object", async () => {
    const { selection } = await openEditor();
    const stalker: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };

    selection.noteHovered(stalker);
    selection.noteHovered({ index: 2, tool: ESceneToolKind.SPAWN });

    expect(selection.hovered).toBe(stalker);

    selection.noteHovered(null);

    expect(selection.hovered).toBeNull();
  });

  it("selects nothing once the objects are read again without the object", async () => {
    const { container, scene, selection } = await openEditor();
    const sessionId: string = scene.scene.sessionId as string;

    selection.select({ index: 2, tool: ESceneToolKind.SPAWN });

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_SCENE_OBJECTS]: (args: { sessionId: string }) =>
        mockSessionSnapshot(
          {
            ...mockSceneObjectsDescription(),
            tools: [mockSceneObjectTool(ESceneToolKind.SPAWN, [mockSceneSpawn(0, "stalker_1", "stalker")])],
          },
          args.sessionId,
          1
        ),
    });
    container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
      isViewRenewed: false,
      revision: 1,
      root: { kind: ESessionRootChange.KEPT },
      sessionId,
      staleParts: [SCENE_OBJECTS_PART],
    });
    await flush();

    expect(scene.objects.snapshot?.revision).toBe(1);
    expect(selection.selected).toBeNull();
  });
});
