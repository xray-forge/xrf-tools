import { beforeEach, describe, expect, it } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import { listMockInvokeCalls, mockInvoke, mockIpcCommands, resetMockChannels, resetMockInvoke } from "@xrf/ipc/testing";
import { ESessionRootChange, SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { ESceneToolKind, SCENE_HELPER_TOOLS, SceneHelperToolEntry } from "@xrf/ipc/types/xrf-app-levels";
import { ViewportEvent } from "@xrf/ipc/types/xrf-app-render";
import { Sphere } from "@xrf/ipc/types/xrf-math";
import { ERenderTextureGroup, ERenderViewportEvent } from "@xrf/ipc/types/xrf-renderer";
import { VisualBounds } from "@xrf/ipc/types/xrf-visual";
import {
  EWorldCamera,
  EWorldCameraCommand,
  EWorldViewportEvent,
  WorldCamera,
  WorldCameraPose,
} from "@xrf/ipc/types/xrf-world";
import { IPoint3 } from "@xrf/math";

import { toLevelFramedGoTo } from "@/core/level/lib/camera/level-camera-frame";
import { toLevelGoToViewpoint } from "@/core/level/lib/camera/level-camera-goto";
import { DEFAULT_LEVEL_CAMERA_OPTIONS } from "@/core/level/lib/camera/level-camera-options";
import { toLevelCameraReading } from "@/core/level/lib/camera/level-camera-reading";
import { ILevelResident } from "@/core/level/lib/render/level-resident";
import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import { toNativeAssetLighting } from "@/core/render/lib/lighting/render-lighting";
import { clearLocalStorage } from "@/core/storage";
import { mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { flush } from "@/fixtures/utils/promise";
import {
  emitSceneViewportEvent,
  MOCK_CLEAN_SCENE_RESIDENT,
  MOCK_SCENE_SESSION,
  MOCK_SCENE_VIEWPORT,
  mockAttachedSceneRender,
} from "@/tools/level-editor/fixtures/scene-render";
import {
  mockSceneDrawDescription,
  mockSceneHelpersDescription,
  mockScenePartResponses,
  mockSelectedSceneDescription,
} from "@/tools/level-editor/fixtures/scene.mocks";
import { getSceneHelperTool } from "@/tools/level-editor/lib/helpers/level-editor-helper-options";
import {
  DEFAULT_LEVEL_EDITOR_LIGHTING,
  toLevelEditorSceneSphere,
} from "@/tools/level-editor/lib/render/level-editor-render";
import { LevelEditorHelpersService } from "@/tools/level-editor/services/level-editor-helpers";

const VIEWPORT: number = MOCK_SCENE_VIEWPORT;
const SESSION: string = MOCK_SCENE_SESSION;
const emit: (event: ViewportEvent) => void = emitSceneViewportEvent;

function toFramedPosition(bounds: VisualBounds): IPoint3 {
  return toLevelGoToViewpoint(
    toLevelFramedGoTo(null, toLevelEditorSceneSphere(bounds.boundingSphere), DEFAULT_LEVEL_CAMERA_OPTIONS.fieldOfView)
  ).position;
}

function expectCameraAt(expected: IPoint3): void {
  const [x, y, z] = lastCamera().position;

  expect(x).toBeCloseTo(expected.x);
  expect(y).toBeCloseTo(expected.y);
  expect(z).toBeCloseTo(expected.z);
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

  // The backend draws a snapshot as it was shown and follows no commit itself.
  it("shows the scene again, concealed until ready, when its snapshot is read again", async () => {
    const { container, service } = await mockAttachedSceneRender();

    emit({ kind: ERenderViewportEvent.LOAD, report: mockRenderLoadReport({ isReady: true, scene: 1 }) });
    await flush();

    expect(service.isRevealed).toBe(true);

    mockInvoke.mockClear();
    mockIpcCommands({
      [ELevelsCommand.RESTORE_SCENE]: () => mockSessionSnapshot(mockSelectedSceneDescription(), SESSION, 1),
      ...mockScenePartResponses({}, 1),
      ...MOCK_CLEAN_SCENE_RESIDENT,
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
    expect(countCalls(ERenderCommand.COMMAND_CAMERA)).toBe(0);

    // A load report of the previously shown renderer scene.
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

  it("draws the scene's helpers, each helped tool switched by its place among the tools", async () => {
    const { container } = await mockAttachedSceneRender();
    const kinds: number = SCENE_HELPER_TOOLS.reduce(
      (bits: number, entry: SceneHelperToolEntry) => (bits | (1 << entry.kind)) >>> 0,
      0
    );

    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)?.options.helpers).toMatchObject({
      hidden: [],
      hovered: null,
      isShown: true,
      kinds,
      selected: [],
    });

    container.get(LevelEditorHelpersService).setHelperTool({ isOn: false, tool: ESceneToolKind.WAY });
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)?.options.helpers.kinds).toBe(
      (kinds & ~(1 << (getSceneHelperTool(ESceneToolKind.WAY) as SceneHelperToolEntry).kind)) >>> 0
    );

    container.get(LevelEditorHelpersService).toggleHelpers();
    await flush();

    expect(listMockInvokeCalls(ERenderCommand.SET_VIEW_OPTIONS).at(-1)?.options.helpers.isShown).toBe(false);
  });

  it("frames the camera on the scene's helpers where it draws no object", async () => {
    await mockAttachedSceneRender({ drawing: { ...mockSceneDrawDescription(), bounds: null } });

    expectCameraAt(toFramedPosition(mockSceneHelpersDescription().bounds as VisualBounds));
  });

  it("frames a sphere asked for keeping the heading the viewport last reported", async () => {
    const { service } = await mockAttachedSceneRender();
    const pose: WorldCameraPose = { position: [0, 10, 0], target: [10, 0, 0] };
    const sphere: Sphere = { center: { x: 3, y: 1, z: -4 }, radius: 2 };

    emit({ kind: EWorldViewportEvent.CAMERA, pose } as ViewportEvent);
    service.frameSphere(sphere);
    await flush();

    expectCameraAt(
      toLevelGoToViewpoint(
        toLevelFramedGoTo(
          toLevelCameraReading(pose),
          toLevelEditorSceneSphere(sphere),
          DEFAULT_LEVEL_CAMERA_OPTIONS.fieldOfView
        )
      ).position
    );
  });

  it("frames the camera on the scene's drawn objects once a scene, and stays as it is revealed", async () => {
    const { service } = await mockAttachedSceneRender();

    expectCameraAt(toFramedPosition(mockSceneDrawDescription().bounds as VisualBounds));
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
