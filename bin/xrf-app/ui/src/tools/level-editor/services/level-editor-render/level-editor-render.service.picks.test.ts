import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import {
  ESceneToolKind,
  SCENE_RENDER_COLLISION_BIT,
  SCENE_RENDER_EDITOR_BIT,
  SceneObjectId,
} from "@xrf/ipc/types/xrf-app-levels";
import { ERenderLevelHit, ERenderSelectionTarget, RenderHelperOptions } from "@xrf/ipc/types/xrf-renderer";

import { clearLocalStorage } from "@/core/storage";
import { flush } from "@/fixtures/utils/promise";
import { MOCK_SCENE_VIEWPORT, mockAttachedSceneRender } from "@/tools/level-editor/fixtures/scene-render";
import { toSceneRenderId } from "@/tools/level-editor/lib/render/scene-render-id";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

/** The fixture's second library object placed. */
const BOX: SceneObjectId = { index: 1, tool: ESceneToolKind.SCENE_OBJECT };

/** The fixture's third spawn point. */
const STALKER: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };

/** Whether the viewport was last told to draw the items group, which the spawn points' models stand in. */
function lastItems(): unknown {
  return listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES).at(-1)?.toggles.isSpawnedItems;
}

/** The render id `object` carries. */
function toId(object: SceneObjectId): number {
  return toSceneRenderId(object) ?? -1;
}

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

  it("selects the scene object a model stands for by its render id, outlining its model", async () => {
    const { container, service } = await mockAttachedSceneRender();

    mockIpcCommands({
      [ERenderCommand.PICK]: { depth: 3, kind: ERenderLevelHit.SPAWN, object: toId(BOX), point: [0, 0, 0] },
    });
    await service.picks.pick({ isThrough: true, point: { x: 10, y: 20 } });

    expect(listMockInvokeCalls(ERenderCommand.PICK).at(-1)).toEqual({
      request: { isThrough: true, x: 10, y: 20 },
      viewport: MOCK_SCENE_VIEWPORT,
    });
    expect(container.get(LevelEditorSelectionService).selectedId).toEqual(BOX);
    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.SET_SELECTION).at(-1)?.selection?.target).toEqual({
        kind: ERenderSelectionTarget.SPAWN,
        object: toId(BOX),
      })
    );
    expect(lastHelpers().selected).toEqual([toId(BOX)]);
  });

  // A spawn point drawn as its model is picked by its model and outlined as one, its volume marked selected too.
  it("selects the spawn point a model or a helper stands for, outlining its model and lifting its helper", async () => {
    const { container, service } = await mockAttachedSceneRender();
    const id: number = toId(STALKER);

    mockIpcCommands({
      [ERenderCommand.PICK]: { depth: 3, kind: ERenderLevelHit.SPAWN, object: id, point: [0, 0, 0] },
    });
    service.picks.click({ isThrough: false, isToggling: false, point: { x: 1, y: 2 } });

    await waitFor(() => expect(container.get(LevelEditorSelectionService).selected?.name).toBe("stalker_2"));
    await flush();

    expect(lastHelpers().selected).toEqual([id]);
    expect(listMockInvokeCalls(ERenderCommand.SET_SELECTION).at(-1)?.selection?.target).toEqual({
      kind: ERenderSelectionTarget.SPAWN,
      object: id,
    });

    container.get(LevelEditorSelectionService).select(null);
    mockIpcCommands({
      [ERenderCommand.PICK]: { depth: 3, kind: ERenderLevelHit.HELPER, object: id, point: [0, 0, 0] },
    });
    await service.picks.pick({ isThrough: false, point: { x: 1, y: 2 } });

    expect(container.get(LevelEditorSelectionService).selectedId).toEqual(STALKER);
  });

  it("selects nothing for a click on nothing, or on an object the scene does not list", async () => {
    const { container, service } = await mockAttachedSceneRender();
    const selection: LevelEditorSelectionService = container.get(LevelEditorSelectionService);

    selection.select(BOX);
    mockIpcCommands({ [ERenderCommand.PICK]: null });
    await service.picks.pick({ isThrough: false, point: { x: 1, y: 1 } });

    expect(selection.selectedId).toBeNull();

    selection.select(BOX);
    mockIpcCommands({
      [ERenderCommand.PICK]: {
        depth: 3,
        kind: ERenderLevelHit.SPAWN,
        object: toId({ index: 9, tool: ESceneToolKind.SCENE_OBJECT }),
        point: [0, 0, 0],
      },
    });
    await service.picks.pick({ isThrough: false, point: { x: 1, y: 1 } });

    expect(selection.selectedId).toBeNull();
  });

  it("lifts the helper under the pointer while helpers are drawn, and nothing once they are off", async () => {
    const { container, service } = await mockAttachedSceneRender();
    const id: number = toId(STALKER);

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

  it("hides an object's models and its helper one by one, by its id and its other models' ids", async () => {
    const { container } = await mockAttachedSceneRender();
    const selection: LevelEditorSelectionService = container.get(LevelEditorSelectionService);

    selection.toggleHidden(STALKER);
    selection.toggleHidden(BOX);
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES).at(-1)?.toggles.hiddenSpawnObjects).toEqual([
      toId(BOX),
      toId(STALKER),
      toId(BOX) + SCENE_RENDER_EDITOR_BIT,
      toId(STALKER) + SCENE_RENDER_EDITOR_BIT,
      toId(BOX) + SCENE_RENDER_COLLISION_BIT,
      toId(STALKER) + SCENE_RENDER_COLLISION_BIT,
    ]);
    expect(lastHelpers().hidden).toEqual([toId(BOX), toId(STALKER)]);

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
        object: toId(STALKER),
        point: [0, 0, 0],
      },
    });
    await service.picks.hover({ x: 4, y: 4 });

    expect(container.get(LevelEditorSelectionService).hovered).toEqual(STALKER);

    await sceneService.close();
    await flush();

    expect(container.get(LevelEditorSelectionService).hovered).toBeNull();
  });

  it("switches the spawn points' models with their helpers in the Show action", async () => {
    const { container } = await mockAttachedSceneRender();
    const helpers: LevelEditorHelpersService = container.get(LevelEditorHelpersService);

    expect(lastItems()).toBe(true);

    helpers.setHelperTool({ isOn: false, tool: ESceneToolKind.SPAWN });
    await flush();

    expect(lastItems()).toBe(false);

    helpers.setHelperTool({ isOn: true, tool: ESceneToolKind.SPAWN });
    helpers.toggleHelpers();
    await flush();

    expect(lastItems()).toBe(false);
    expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES).at(-1)?.toggles.isSpawnedProps).toBe(true);
  });
});
