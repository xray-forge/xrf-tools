import { beforeEach, describe, expect, it } from "@jest/globals";
import { Container, EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { mockInvoke, mockIpcCommands, resetMockInvoke } from "@xrf/ipc/testing";
import { ESessionRootChange, SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { ESceneToolKind, SCENE_BINARY_TOOLS_PART, SCENE_OBJECTS_PART } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";

import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import {
  mockSceneObject,
  mockSceneObjectsDescription,
  mockSceneObjectTool,
  mockScenePartResponses,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";

import { LevelEditorSceneService } from "./level-editor-scene.service";

interface IOpenedScene {
  container: Container;
  service: LevelEditorSceneService;
  sessionId: string;
}

function countCalls(command: string): number {
  return mockInvoke.mock.calls.filter(([name]) => name === command).length;
}

async function openScene(): Promise<IOpenedScene> {
  const container: Container = mockContainer([LevelEditorSceneService]);
  const service: LevelEditorSceneService = container.get(LevelEditorSceneService);

  mockIpcCommands({
    [ELevelsCommand.RESTORE_SCENE]: null,
    [ELevelsCommand.OPEN_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(),
  });

  container.provision();
  await flush();
  await service.open({
    engine: EXrayEngineChoice.AUTO,
    gameRoots: null,
    isDltx: false,
    objectsRoots: null,
    source: { path: "C:\\rawdata\\levels\\labx8.level" },
  });
  await flush();

  return { container, service, sessionId: service.scene.sessionId as string };
}

function notify(container: Container, change: SessionChange): void {
  container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, change);
}

describe("LevelEditorSceneService changes", () => {
  beforeEach(() => {
    resetMockInvoke();
  });

  it("reads the objects again alone when a change names them stale", async () => {
    const { container, service, sessionId } = await openScene();
    const binaryReads: number = countCalls(ELevelsCommand.DESCRIBE_SCENE_BINARY_TOOLS);

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_SCENE_OBJECTS]: (args: { sessionId: string }) =>
        mockSessionSnapshot(
          {
            problems: [],
            tools: [mockSceneObjectTool(ESceneToolKind.SCENE_OBJECT, [mockSceneObject({ name: "moved" })])],
          },
          args.sessionId,
          1
        ),
    });
    notify(container, {
      isViewRenewed: false,
      revision: 1,
      root: { kind: ESessionRootChange.KEPT },
      sessionId,
      staleParts: [SCENE_OBJECTS_PART],
    });
    await flush();

    expect(service.objects.value?.tools[0]?.objects[0]?.name).toBe("moved");
    expect(service.objects.snapshot?.revision).toBe(1);
    expect(countCalls(ELevelsCommand.DESCRIBE_SCENE_BINARY_TOOLS)).toBe(binaryReads);
    expect(service.isStale).toBe(false);
  });

  it("reads nothing again for another scene's change", async () => {
    const { container, service } = await openScene();
    const reads: number = countCalls(ELevelsCommand.DESCRIBE_SCENE_OBJECTS);

    notify(container, {
      isViewRenewed: false,
      revision: 1,
      root: { kind: ESessionRootChange.KEPT },
      sessionId: "another",
      staleParts: [SCENE_OBJECTS_PART, SCENE_BINARY_TOOLS_PART],
    });
    await flush();

    expect(countCalls(ELevelsCommand.DESCRIBE_SCENE_OBJECTS)).toBe(reads);
    expect(service.objects.value).toEqual(mockSceneObjectsDescription());
  });

  // As when the `.level` is edited on disk.
  it("reads a scene its session derived anew, and both its parts", async () => {
    const { container, service, sessionId } = await openScene();

    mockIpcCommands({
      [ELevelsCommand.RESTORE_SCENE]: () =>
        mockSessionSnapshot(mockSelectedSceneDescription({ guid: "0:1" }), sessionId, 1),
      ...mockScenePartResponses({}, 1),
    });
    notify(container, {
      isViewRenewed: false,
      revision: 0,
      root: { kind: ESessionRootChange.DERIVING },
      sessionId,
      staleParts: [],
    });
    await flush();

    expect(service.isStale).toBe(true);

    notify(container, {
      isViewRenewed: false,
      revision: 1,
      root: { kind: ESessionRootChange.DERIVED },
      sessionId,
      staleParts: [],
    });
    await flush();

    expect(service.scene.value?.guid).toBe("0:1");
    expect(service.isStale).toBe(false);
    expect(service.objects.snapshot?.revision).toBe(1);
    expect(service.binaryTools.snapshot?.revision).toBe(1);
  });

  it("marks a scene whose own files moved stale, reading nothing again", async () => {
    const { container, service, sessionId } = await openScene();
    const reads: number = countCalls(ELevelsCommand.DESCRIBE_SCENE_OBJECTS);

    notify(container, {
      isViewRenewed: false,
      revision: 0,
      root: { kind: ESessionRootChange.STALE },
      sessionId,
      staleParts: [SCENE_OBJECTS_PART],
    });
    await flush();

    expect(service.isStale).toBe(true);
    expect(countCalls(ELevelsCommand.DESCRIBE_SCENE_OBJECTS)).toBe(reads);
    expect(service.objects.value).toEqual(mockSceneObjectsDescription());
  });
});
