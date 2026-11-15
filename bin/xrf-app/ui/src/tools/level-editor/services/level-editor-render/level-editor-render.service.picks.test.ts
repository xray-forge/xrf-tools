import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { ESceneToolKind, SceneObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { ERenderLevelHit, ERenderSelectionTarget, RenderHelperOptions } from "@xrf/ipc/types/xrf-renderer";

import { clearLocalStorage } from "@/core/storage";
import { flush } from "@/fixtures/utils/promise";
import { MOCK_SCENE_VIEWPORT, mockAttachedSceneRender } from "@/tools/level-editor/fixtures/scene-render";
import { toSceneHelperId } from "@/tools/level-editor/lib/helpers/scene-helper-id";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** The fixture's second library object placed. */
const BOX: SceneObjectId = { index: 1, tool: ESceneToolKind.SCENE_OBJECT };

/** The fixture's third spawn point. */
const STALKER: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };

/** The helpers the viewport was last told to draw. */
function lastHelpers(): RenderHelperOptions {
  return listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)?.options.helpers as RenderHelperOptions;
}

describe("LevelEditorRenderPicks", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  it("selects the scene object a model stands for by its place among them, outlining its model", async () => {
    const { container, service } = await mockAttachedSceneRender();

    mockIpcCommands({ [ERenderCommand.PICK]: { depth: 3, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] } });
    await service.picks.pick({ isThrough: true, point: { x: 10, y: 20 } });

    expect(listMockInvokeCalls(ERenderCommand.PICK).at(-1)).toEqual({
      request: { isThrough: true, x: 10, y: 20 },
      viewport: MOCK_SCENE_VIEWPORT,
    });
    expect(container.get(LevelEditorSelectionService).selectedId).toEqual(BOX);
    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.SET_SELECTION).at(-1)?.selection?.target).toEqual({
        kind: ERenderSelectionTarget.SPAWN,
        object: 1,
      })
    );
    expect(lastHelpers().selected).toEqual([]);
  });

  it("selects the scene object a helper stands for, lifting its helper rather than outlining a model", async () => {
    const { container, service } = await mockAttachedSceneRender();
    const id: number = toSceneHelperId(STALKER) as number;

    mockIpcCommands({
      [ERenderCommand.PICK]: { depth: 3, kind: ERenderLevelHit.HELPER, object: id, point: [0, 0, 0] },
    });
    service.picks.click({ isThrough: false, isToggling: false, point: { x: 1, y: 2 } });

    await waitFor(() => expect(container.get(LevelEditorSelectionService).selected?.name).toBe("stalker_2"));
    await flush();

    expect(lastHelpers().selected).toEqual([id]);
    expect(listMockInvokeCalls(ERenderCommand.SET_SELECTION).at(-1)?.selection).toBeNull();
  });

  it("selects nothing for a click on nothing, or on an object the scene does not list", async () => {
    const { container, service } = await mockAttachedSceneRender();
    const selection: LevelEditorSelectionService = container.get(LevelEditorSelectionService);

    selection.select(BOX);
    mockIpcCommands({ [ERenderCommand.PICK]: null });
    await service.picks.pick({ isThrough: false, point: { x: 1, y: 1 } });

    expect(selection.selectedId).toBeNull();

    selection.select(BOX);
    mockIpcCommands({ [ERenderCommand.PICK]: { depth: 3, kind: ERenderLevelHit.SPAWN, object: 9, point: [0, 0, 0] } });
    await service.picks.pick({ isThrough: false, point: { x: 1, y: 1 } });

    expect(selection.selectedId).toBeNull();
  });

  it("lifts the helper under the pointer while helpers are drawn, and nothing once they are off", async () => {
    const { container, service } = await mockAttachedSceneRender();
    const id: number = toSceneHelperId(STALKER) as number;

    mockIpcCommands({
      [ERenderCommand.PICK]: { depth: 3, kind: ERenderLevelHit.HELPER, object: id, point: [0, 0, 0] },
    });
    await service.picks.hover({ x: 4, y: 4 });
    await flush();

    expect(container.get(LevelEditorSelectionService).hovered).toEqual(STALKER);
    expect(lastHelpers().hovered).toBe(id);

    container.get(LevelEditorHelpersService).toggleHelpers();
    await waitFor(() => expect(container.get(LevelEditorSelectionService).hovered).toBeNull());

    const picks: number = listMockInvokeCalls(ERenderCommand.PICK).length;

    service.picks.rest({ x: 5, y: 5 });
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.PICK)).toHaveLength(picks);
  });

  it("hides a scene object's model and a helped object's helper one by one", async () => {
    const { container } = await mockAttachedSceneRender();
    const selection: LevelEditorSelectionService = container.get(LevelEditorSelectionService);

    selection.toggleHidden(STALKER);
    selection.toggleHidden(BOX);
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES).at(-1)?.toggles.hiddenSpawnObjects).toEqual([1]);
    expect(lastHelpers().hidden).toEqual([toSceneHelperId(STALKER)]);

    selection.revealAll();
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES).at(-1)?.toggles.hiddenSpawnObjects).toEqual([]);
    expect(lastHelpers().hidden).toEqual([]);
  });

  it("lets go of what is hovered once the scene shown is let go", async () => {
    const { container, sceneService, service } = await mockAttachedSceneRender();

    mockIpcCommands({
      [ERenderCommand.PICK]: {
        depth: 3,
        kind: ERenderLevelHit.HELPER,
        object: toSceneHelperId(STALKER) as number,
        point: [0, 0, 0],
      },
    });
    await service.picks.hover({ x: 4, y: 4 });

    expect(container.get(LevelEditorSelectionService).hovered).toEqual(STALKER);

    await sceneService.close();
    await flush();

    expect(container.get(LevelEditorSelectionService).hovered).toBeNull();
  });
});
