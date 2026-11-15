import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { Container, EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import {
  getMockChannels,
  IpcCommandMocks,
  listMockInvokeCalls,
  MockChannel,
  mockInvoke,
  mockIpcCommands,
  resetMockChannels,
  resetMockInvoke,
} from "@xrf/ipc/testing";
import { ESessionRootChange, SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { ViewportEvent } from "@xrf/ipc/types/xrf-app-render";
import { ERenderTextureGroup, ERenderViewportEvent } from "@xrf/ipc/types/xrf-renderer";
import { VisualBounds } from "@xrf/ipc/types/xrf-visual";
import { EWorldCamera, EWorldCameraCommand, WorldCamera } from "@xrf/ipc/types/xrf-world";
import { IPoint3 } from "@xrf/math";

import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { DEFAULT_LEVEL_CAMERA_OPTIONS } from "@/core/level/lib/camera/level-camera-options";
import { ILevelResident } from "@/core/level/lib/render/level-resident";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import { toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import { clearLocalStorage } from "@/core/storage";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { flush } from "@/fixtures/utils/promise";
import {
  mockSceneDrawDescription,
  mockScenePartResponses,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import {
  DEFAULT_LEVEL_EDITOR_LIGHTING,
  toLevelEditorSceneSphere,
} from "@/tools/level-editor/lib/render/level-editor-render";
import { LevelEditorSceneService } from "@/tools/level-editor/services/level-editor-scene";

import { LevelEditorRenderService } from "./level-editor-render.service";

/** The viewport the mocked renderer attaches. */
const VIEWPORT: number = 5;

/** The session the fixture scene is restored in. */
const SESSION: string = "fixture-session";

/** What the renderer answers of a resident scene it drew whole. */
const CLEAN_RESIDENT_RESPONSES: IpcCommandMocks = {
  [ERenderCommand.DESCRIBE_PROBLEMS]: { models: [], sectors: [], skipped: [] },
  [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [],
};

interface IAttachedSceneRender {
  container: Container;
  sceneService: LevelEditorSceneService;
  service: LevelEditorRenderService;
}

/**
 * The fixture scene restored and provisioned as the editor provisions it, its render service attached; each show
 * answers a scene of its own, as the renderer starts one for every show of a viewport it alone draws.
 */
async function mockAttachedSceneRender(): Promise<IAttachedSceneRender> {
  let scenes: number = 0;

  mockIpcCommands({
    [ELevelsCommand.RESTORE_SCENE]: mockSessionResponse(mockSelectedSceneDescription()),
    ...mockScenePartResponses(),
    [ERenderCommand.ATTACH_VIEWPORT]: VIEWPORT,
    [ELevelsCommand.SHOW_SCENE]: () => (scenes += 1),
    ...CLEAN_RESIDENT_RESPONSES,
  });

  const container: Container = mockContainer([
    LevelEditorSceneService,
    LevelViewService,
    LevelViewportService,
    LevelEditorRenderService,
  ]);
  const sceneService: LevelEditorSceneService = container.get(LevelEditorSceneService);

  container.provision();
  await waitFor(() => expect(sceneService.drawing.value).not.toBeNull());

  const service: LevelEditorRenderService = container.get(LevelEditorRenderService);

  service.attach(document.createElement("div"));
  await waitFor(() => expect(listMockInvokeCalls(ELevelsCommand.SHOW_SCENE)).toHaveLength(1));

  return { container, sceneService, service };
}

function emit(event: ViewportEvent): void {
  (getMockChannels()[0] as MockChannel<ViewportEvent>).onmessage(event);
}

function countCalls(command: string): number {
  return mockInvoke.mock.calls.filter(([name]) => name === command).length;
}

function lastCamera(): WorldCamera {
  return listMockInvokeCalls(ERenderCommand.SET_CAMERA).at(-1)?.camera as WorldCamera;
}

describe("LevelEditorRenderService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    clearLocalStorage();
  });

  it("shows the open scene once adopted, under the editor's light, its objects drawn", async () => {
    const { service } = await mockAttachedSceneRender();

    expect(listMockInvokeCalls(ELevelsCommand.SHOW_SCENE)).toEqual([{ sessionId: SESSION, viewport: VIEWPORT }]);
    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)?.options.asset.lighting).toEqual(
      toNativeAssetLighting(DEFAULT_LEVEL_EDITOR_LIGHTING)
    );
    expect(listMockInvokeCalls(ERenderCommand.SET_WORLD_TOGGLES).at(-1)?.toggles).toMatchObject({
      hiddenSpawnObjects: [],
      isSpawnedProps: true,
    });
    expect(lastCamera().kind).toBe(EWorldCamera.FLY);
    expect(service.isRevealed).toBe(false);
  });

  // The renderer answers for its textures and models as it does for any source; the scene's part is not read again.
  it("reveals the scene once it is resident whole, and asks the renderer then what it could not draw", async () => {
    const { service } = await mockAttachedSceneRender();
    const reads: number = countCalls(ELevelsCommand.DESCRIBE_SCENE_DRAW);

    mockIpcCommands({
      [ERenderCommand.DESCRIBE_PROBLEMS]: {
        models: [{ name: "props\\box", reason: "Library object 'props\\box' does not compile: no surfaces" }],
        sectors: [],
        skipped: [],
      },
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [
        { group: ERenderTextureGroup.SURFACES, reference: "terrain\\terrain_zaton", state: { kind: "missing" } },
      ],
    });
    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: false, scene: 1 }) });
    await flush();

    expect(service.isRevealed).toBe(false);
    expect(service.resident.state.isIdle).toBe(true);

    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: 1 }) });
    await waitFor(() => expect(service.resident.state.isReady).toBe(true));

    const resident: ILevelResident = service.resident.value as ILevelResident;

    expect(service.isRevealed).toBe(true);
    expect(resident.problems.models).toEqual([
      { name: "props\\box", reason: "Library object 'props\\box' does not compile: no surfaces" },
    ]);
    expect(resident.textures.problems).toEqual([
      {
        group: ERenderTextureGroup.SURFACES,
        reason: "Nothing in the mounted roots answers to it",
        reference: "terrain\\terrain_zaton",
      },
    ]);
    // A scene has no sectors to measure.
    expect(resident.surfaces.size).toBe(0);
    expect(countCalls(ERenderCommand.MEASURE_SURFACES)).toBe(0);
    expect(countCalls(ELevelsCommand.DESCRIBE_SCENE_DRAW)).toBe(reads);
  });

  it("forgets what the renderer said of the scene shown once it is let go", async () => {
    const { sceneService, service } = await mockAttachedSceneRender();

    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: 1 }) });
    await waitFor(() => expect(service.resident.state.isReady).toBe(true));

    await sceneService.close();
    await flush();

    expect(service.resident.state.isIdle).toBe(true);
    expect(service.resident.value).toBeNull();
  });

  // The backend draws a snapshot as it was shown: the page shows the scene's session again once it is read again.
  it("shows the scene again, concealed until ready, when its snapshot is read again", async () => {
    const { container, service } = await mockAttachedSceneRender();

    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: 1 }) });
    await flush();

    expect(service.isRevealed).toBe(true);

    mockInvoke.mockClear();
    mockIpcCommands({
      [ELevelsCommand.RESTORE_SCENE]: () => mockSessionSnapshot(mockSelectedSceneDescription(), SESSION, 1),
      ...mockScenePartResponses({}, 1),
      ...CLEAN_RESIDENT_RESPONSES,
    });
    container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
      isViewRenewed: false,
      revision: 1,
      root: { kind: ESessionRootChange.DERIVED },
      sessionId: SESSION,
      staleParts: [],
    });
    await waitFor(() => expect(listMockInvokeCalls(ELevelsCommand.SHOW_SCENE)).toHaveLength(1));

    expect(listMockInvokeCalls(ELevelsCommand.SHOW_SCENE)).toEqual([{ sessionId: SESSION, viewport: VIEWPORT }]);
    expect(service.isRevealed).toBe(false);
    // The camera stays where it has flown: the same scene, read again.
    expect(countCalls(ERenderCommand.COMMAND_CAMERA)).toBe(0);

    // The scene shown before is not the one read again.
    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: 1 }) });
    await flush();

    expect(service.isRevealed).toBe(false);

    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: 2 }) });
    await flush();

    expect(service.isRevealed).toBe(true);
  });

  it("clears the viewport once the scene closes", async () => {
    const { sceneService } = await mockAttachedSceneRender();

    await sceneService.close();
    await flush();

    expect(listMockInvokeCalls(ELevelsCommand.SHOW_SCENE).at(-1)).toEqual({ sessionId: null, viewport: VIEWPORT });
  });

  it("frames the camera on the scene's drawn objects once a scene, and stays as it is revealed", async () => {
    const { service } = await mockAttachedSceneRender();

    const bounds: VisualBounds = mockSceneDrawDescription().bounds as VisualBounds;
    const expected: IPoint3 = toLevelGoToViewpoint(
      toLevelFramedGoTo(null, toLevelEditorSceneSphere(bounds), DEFAULT_LEVEL_CAMERA_OPTIONS.fieldOfView)
    ).position;
    const [x, y, z] = lastCamera().position;

    expect(x).toBeCloseTo(expected.x);
    expect(y).toBeCloseTo(expected.y);
    expect(z).toBeCloseTo(expected.z);
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA).at(-1)).toEqual({
      command: { kind: EWorldCameraCommand.RESET },
      viewport: VIEWPORT,
    });

    mockInvoke.mockClear();
    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: 1 }) });
    await waitFor(() => expect(service.resident.state.isReady).toBe(true));
    await flush();

    expect(countCalls(ERenderCommand.SET_CAMERA)).toBe(0);
    expect(countCalls(ERenderCommand.COMMAND_CAMERA)).toBe(0);
  });
});
