import { beforeEach, describe, expect, it } from "@jest/globals";
import { isObservableProp } from "@wirestate/mobx";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ESessionsCommand } from "@xrf/ipc/commands/sessions";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { SceneOpenRequest } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { mockInjectedService } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import {
  mockSceneDrawDescription,
  mockSceneHelpersDescription,
  mockSceneObjectsDescription,
  mockScenePartResponses,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";

import { LevelEditorSceneService } from "./level-editor-scene.service";

const REQUEST: SceneOpenRequest = {
  engine: EXrayEngineChoice.AUTO,
  gameRoots: null,
  isDltx: false,
  objectsRoots: null,
  source: { path: "C:\\rawdata\\levels\\labx8.level" },
};
const OTHER_REQUEST: SceneOpenRequest = {
  engine: EXrayEngineChoice.AUTO,
  gameRoots: null,
  isDltx: false,
  objectsRoots: ["C:\\sdk\\rawdata\\objects"],
  source: { path: "C:\\rawdata\\levels\\zaton.level" },
};

function countCalls(command: string): number {
  return mockInvoke.mock.calls.filter(([name]) => name === command).length;
}

/** Arms an open of the fixture scene and the reads of its two parts. */
function armScene(): void {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(),
  });
}

function armFailedOpen(message: string): void {
  mockIpcCommands({
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(() => {
      throw new Error(message);
    }),
  });
}

describe("LevelEditorSceneService", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("applies its mobx annotations", () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    expect(isObservableProp(service, "opening")).toBe(true);
    expect(isObservableProp(service, "isStale")).toBe(true);
  });

  // The open lists the part files alone: what each tool holds is read apart from it, side by side.
  it("opens a scene, then reads its objects, its binary tools' counts, what its picture and helpers need", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    armScene();

    await service.open(REQUEST);
    await flush();

    expect(service.scene.value?.name).toBe("labx8");
    expect(service.objects.value).toEqual(mockSceneObjectsDescription());
    expect(service.binaryTools.value?.tools).toHaveLength(1);
    expect(service.drawing.value).toEqual(mockSceneDrawDescription());
    expect(service.helpers.value).toEqual(mockSceneHelpersDescription());
    expect(mockInvoke.mock.calls.map(([name]) => name)).toEqual([
      ESessionsCommand.BIND,
      ELevelsCommand.OPEN_SCENE,
      ELevelsCommand.DESCRIBE_SCENE_OBJECTS,
      ELevelsCommand.DESCRIBE_SCENE_BINARY_TOOLS,
      ELevelsCommand.DESCRIBE_SCENE_DRAW,
      ELevelsCommand.DESCRIBE_SCENE_HELPERS,
    ]);
    expect(listMockInvokeCalls(ELevelsCommand.OPEN_SCENE)[0]?.request).toEqual(REQUEST);
  });

  it("names the scene it is opening until it is open", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    armScene();

    const opened: Promise<void> = service.open(REQUEST);

    expect(service.opening).toEqual(REQUEST.source);

    await opened;

    expect(service.opening).toBeNull();
  });

  // A refresh re-provisions the service while the backend keeps the scene, which is why it keeps it.
  it("takes back the scene the backend still has open, without opening it again", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    mockIpcCommands({
      [ELevelsCommand.RESTORE_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
      ...mockScenePartResponses(),
    });

    await service.onProvision();
    await flush();

    expect(service.scene.isRestored).toBe(true);
    expect(service.scene.value?.guid).toBe(mockSelectedSceneDescription().guid);
    expect(service.objects.value).not.toBeNull();
    expect(countCalls(ELevelsCommand.OPEN_SCENE)).toBe(0);
  });

  it("reports itself restored when there is nothing to restore", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    mockIpcCommands({ [ELevelsCommand.RESTORE_SCENE]: null });

    await service.onProvision();

    expect(service.scene.isRestored).toBe(true);
    expect(service.scene.value).toBeNull();
    expect(countCalls(ELevelsCommand.DESCRIBE_SCENE_OBJECTS)).toBe(0);
  });

  it("forgets the scene and its parts once closed", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    armScene();

    await service.open(REQUEST);
    await flush();
    await service.close();

    expect(service.scene.value).toBeNull();
    expect(service.objects.value).toBeNull();
    expect(service.binaryTools.value).toBeNull();
    expect(service.drawing.value).toBeNull();
    expect(listMockInvokeCalls(ELevelsCommand.CLOSE_SCENE)).toHaveLength(1);
  });

  // Showing the scene it was asked to replace, with no word, reads as the open having worked.
  it("closes the open scene when another fails to open, and holds the failure", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    armScene();

    await service.open(REQUEST);
    await flush();

    armFailedOpen("The .level holds no [version]");

    await service.open(OTHER_REQUEST);

    expect(service.scene.value).toBeNull();
    expect(service.scene.error?.message).toBe("The .level holds no [version]");
    expect(service.objects.value).toBeNull();
    expect(countCalls(ELevelsCommand.CLOSE_SCENE)).toBe(1);
  });

  it("opens the last scene asked for again on a retry", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    armFailedOpen("The file is locked");

    await service.open(OTHER_REQUEST);

    expect(service.scene.state.isFailed).toBe(true);

    armScene();

    await service.retry();

    expect(service.scene.value?.name).toBe("labx8");
    expect(listMockInvokeCalls(ELevelsCommand.OPEN_SCENE).at(-1)?.request).toEqual(OTHER_REQUEST);
  });

  it("lets the scene go at once when cleared", async () => {
    const { service } = mockInjectedService(LevelEditorSceneService);

    armScene();

    await service.open(REQUEST);
    await flush();

    service.clear();

    expect(service.scene.value).toBeNull();
    expect(service.objects.value).toBeNull();
    expect(service.binaryTools.value).toBeNull();
  });
});
