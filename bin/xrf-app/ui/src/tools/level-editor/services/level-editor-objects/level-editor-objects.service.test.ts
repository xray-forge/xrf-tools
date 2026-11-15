import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { ESceneToolKind, SceneObjectId } from "@xrf/ipc/types/xrf-app-levels";
import { Sphere } from "@xrf/ipc/types/xrf-math";
import { WorldCamera } from "@xrf/ipc/types/xrf-world";

import { KeybindCommandsService } from "@/core/commands";
import { FRAME_LEVEL_SELECTION_KEYBIND_COMMAND } from "@/core/level/commands";
import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { DEFAULT_LEVEL_CAMERA_OPTIONS } from "@/core/level/lib/camera/level-camera-options";
import { clearLocalStorage } from "@/core/storage";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { flush } from "@/fixtures/utils/promise";
import { MOCK_SCENE_SESSION, mockAttachedSceneRender } from "@/tools/level-editor/fixtures/scene-render";
import { toLevelEditorSceneSphere } from "@/tools/level-editor/lib/render/level-editor-render";
import { LevelEditorSelectionService } from "@/tools/level-editor/services/level-editor-selection";

import { LevelEditorObjectsService } from "./level-editor-objects.service";

/** The fixture's third spawn point. */
const STALKER: SceneObjectId = { index: 2, tool: ESceneToolKind.SPAWN };

/** Where the backend says the fixture's objects stand. */
const SPHERE: Sphere = { center: { x: 30, y: 2, z: -6 }, radius: 1.5 };

/** Answers every locate with `sphere`, for the session asked about. */
function mockLocated(sphere: Sphere | null): void {
  mockIpcCommands({
    [ELevelsCommand.LOCATE_SCENE_OBJECT]: (args: { sessionId: string }) => mockSessionSnapshot(sphere, args.sessionId),
  });
}

function lastCamera(): WorldCamera {
  return listMockInvokeCalls(ERenderCommand.SET_CAMERA).at(-1)?.camera as WorldCamera;
}

describe("LevelEditorObjectsService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  it("frames an object by the sphere the backend says it is drawn in", async () => {
    const { container } = await mockAttachedSceneRender();

    mockLocated(SPHERE);
    await container.get(LevelEditorObjectsService).goTo(STALKER);

    expect(listMockInvokeCalls(ELevelsCommand.LOCATE_SCENE_OBJECT)).toEqual([
      { object: STALKER, sessionId: MOCK_SCENE_SESSION },
    ]);

    const [x, y, z] = lastCamera().position;
    // No camera reported yet: framed facing as a camera does before it reports.
    const expected = toLevelGoToViewpoint(
      toLevelFramedGoTo(null, toLevelEditorSceneSphere(SPHERE), DEFAULT_LEVEL_CAMERA_OPTIONS.fieldOfView)
    ).position;

    expect(x).toBeCloseTo(expected.x);
    expect(y).toBeCloseTo(expected.y);
    expect(z).toBeCloseTo(expected.z);
  });

  it("leaves the camera where it is for an object that draws nothing, or that the backend cannot place", async () => {
    const { container } = await mockAttachedSceneRender();
    const service: LevelEditorObjectsService = container.get(LevelEditorObjectsService);
    const cameras: number = listMockInvokeCalls(ERenderCommand.SET_CAMERA).length;

    mockLocated(null);
    await service.goTo({ index: 0, tool: ESceneToolKind.SECTOR });

    mockIpcCommands({
      [ELevelsCommand.LOCATE_SCENE_OBJECT]: () => {
        throw new Error("no scene is held under the session");
      },
    });
    await service.goTo(STALKER);

    expect(listMockInvokeCalls(ERenderCommand.SET_CAMERA)).toHaveLength(cameras);
  });

  it("selects and frames an object focused, as an Outliner row's activation does", async () => {
    const { container } = await mockAttachedSceneRender();

    mockLocated(SPHERE);
    container.get(LevelEditorObjectsService).focus(STALKER);

    expect(container.get(LevelEditorSelectionService).selectedId).toEqual(STALKER);
    await waitFor(() => expect(listMockInvokeCalls(ELevelsCommand.LOCATE_SCENE_OBJECT)).toHaveLength(1));
  });

  it("frames the selection with `F` while something is selected", async () => {
    const { container } = await mockAttachedSceneRender();
    const commandsService: KeybindCommandsService = container.get(KeybindCommandsService);
    const selection: LevelEditorSelectionService = container.get(LevelEditorSelectionService);

    expect(commandsService.isAvailable(FRAME_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(false);

    selection.select(STALKER);
    mockLocated(SPHERE);

    expect(commandsService.isAvailable(FRAME_LEVEL_SELECTION_KEYBIND_COMMAND)).toBe(true);

    container.get(LevelEditorObjectsService).frameSelection();
    await flush();

    expect(listMockInvokeCalls(ELevelsCommand.LOCATE_SCENE_OBJECT)).toEqual([
      { object: STALKER, sessionId: MOCK_SCENE_SESSION },
    ]);
  });
});
