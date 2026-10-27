import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { EventBus } from "@wirestate/core";
import { ELevelsCommand } from "@xrf/ipc/commands/levels";
import { ERenderCommand } from "@xrf/ipc/commands/render";
import { SESSION_CHANGE_EVENT } from "@xrf/ipc/session";
import {
  getMockChannels,
  listMockInvokeCalls,
  mockInvoke,
  mockIpcCommands,
  resetMockChannels,
  resetMockInvoke,
} from "@xrf/ipc/testing";
import { SessionChange } from "@xrf/ipc/types/xrf-app-core";
import { LEVEL_GEOMETRY_PART, LEVEL_MAP_PART } from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";
import {
  ERenderAntialiasing,
  ERenderDlssAvailability,
  ERenderLevelHit,
  ERenderTextureFiltering,
  ERenderTextureGroup,
  ERenderViewportEvent,
  RENDER_DEFAULT_PERIODIC_BUDGET,
  RenderFrameReport,
  RenderLevelHit,
} from "@xrf/ipc/types/xrf-renderer";
import {
  EWorldCamera,
  EWorldCameraCommand,
  EWorldViewportEvent,
  EWorldWeatherPlay,
  WorldCamera,
  WorldSurfaceSpan,
} from "@xrf/ipc/types/xrf-world";

import { createRoots } from "@/core/assets/lib";
import {
  emitViewportEvent,
  LEVEL_SCENE,
  LEVEL_VIEWPORT,
  mockAttachedLevelRender,
} from "@/core/level/fixtures/level-render";
import { mockSelectedLevelDescription } from "@/core/level/fixtures/level.mocks";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS } from "@/core/level/lib/weather/level-sun-shafts-options";
import { DEFAULT_LEVEL_WEATHER_CONTROL } from "@/core/level/lib/weather/level-weather-control";
import { toLevelWeatherMemoryKey, writeLevelWeatherMemory } from "@/core/level/lib/weather/level-weather-memory";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import { EMPTY_RENDER_FRAME_PHASES } from "@/core/render/lib/native/native-frame-report";
import { DEFAULT_RENDER_GRAPH_SETTINGS } from "@/core/render/lib/settings/render-graph-settings";
import { clearLocalStorage } from "@/core/storage";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { flush } from "@/fixtures/utils/promise";

const SPAN: WorldSurfaceSpan = { uMax: 2, uMin: 0, vMax: 1, vMin: -1 };

const REPORT: RenderFrameReport = {
  adapter: "Test GPU",
  backend: "d3d12",
  cpuTime: 0.8,
  cpuTimes: [0.7, 0.8, 0.9],
  frameTime: 6.25,
  frameTimeMax: 9,
  frameTimes: [6, 6.5, 9],
  gpuTime: 5.5,
  gpuTimes: [5, 5.5, 6],
  graph: null,
  groups: [{ finish: 0.4, name: "scene", record: 0.1 }],
  framesPerSecond: 160,
  height: 600,
  hasRayQueries: true,
  dlss: ERenderDlssAvailability.UNBUILT,
  rayReconstruction: ERenderDlssAvailability.UNBUILT,
  outputWidth: 534,
  outputHeight: 400,
  antialiasing: ERenderAntialiasing.TAA,
  isRayReconstructed: false,
  isGpuTimed: true,
  phases: EMPTY_RENDER_FRAME_PHASES,
  passes: [
    { gpuTime: 1.25, name: "g-buffer" },
    { gpuTime: 0.5, name: "sun" },
  ],
  memory: {
    allocated: null,
    budget: null,
    peakAllocated: null,
    reserved: null,
    sections: [],
    total: null,
    used: null,
  },
  meter: null,
  lights: {
    atlas: { capacity: 16_777_216, used: 4_194_304 },
    dropped: 3,
    excess: 0,
    fullClusters: 2,
    inView: 12,
    shadowed: 4,
    volumetric: 2,
  },
  particles: { drawn: 9, effects: 14, particles: 210, simulated: 11, simulationTime: 0.3 },
  grass: { capacity: 4096, drawn: 1200 },
  renderHeight: 400,
  renderWidth: 534,
  sectorTime: 1.5,
  staticDraws: {
    clusters: { capacity: 4096, used: 3000 },
    commands: 13,
    keptClusters: 1200,
    keptTriangles: 90_000,
    lods: { capacity: 64, used: 40 },
    occludedClusters: 300,
    occludedTriangles: 20_000,
    places: { capacity: 2048, used: 1500 },
    rows: { capacity: 512, used: 200 },
    slots: { capacity: 1024, used: 900 },
    surfaceList: { capacity: 8192, used: 1200 },
  },
  width: 800,
};

describe("LevelRenderService", () => {
  beforeEach(() => {
    resetMockInvoke();
    resetMockChannels();
    mockInvoke.mockClear();
    clearLocalStorage();
  });

  it("attaches a native viewport to its window and stands the camera at the level's start", async () => {
    await mockAttachedLevelRender();

    expect(listMockInvokeCalls(ERenderCommand.ATTACH_VIEWPORT)).toEqual([{ events: getMockChannels()[0] }]);
    expect(listMockInvokeCalls(ERenderCommand.CONFIGURE)).toEqual([
      {
        settings: {
          backend: null,
          frameRate: { isVsync: true, limit: null },
          gpu: null,
          graph: DEFAULT_RENDER_GRAPH_SETTINGS,
          isGpuTimed: false,
          periodicBudget: RENDER_DEFAULT_PERIODIC_BUDGET,
          textureFiltering: ERenderTextureFiltering.ANISOTROPIC16,
          textureLod: 0,
        },
      },
    ]);

    const camera: WorldCamera = listMockInvokeCalls(ERenderCommand.SET_CAMERA).at(-1)?.camera as WorldCamera;

    expect(listMockInvokeCalls(ERenderCommand.SET_CAMERA).at(-1)?.viewport).toBe(LEVEL_VIEWPORT);
    expect(camera.kind).toBe(EWorldCamera.FLY);
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA).at(-1)).toEqual({
      command: { kind: EWorldCameraCommand.RESET },
      viewport: LEVEL_VIEWPORT,
    });
  });

  it("shows the open level in its viewport and reveals it once the renderer has it resident", async () => {
    const { container } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toEqual([
      { sessionId: expect.any(String), viewport: LEVEL_VIEWPORT },
    ]);
    expect(viewport.isRevealed).toBe(false);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({
        bytes: 10,
        isReady: false,
        scene: LEVEL_SCENE,
        sectors: 1,
        sectorsTotal: 2,
        textures: 0,
        texturesTotal: 3,
      }),
    });
    expect(viewport.isRevealed).toBe(false);

    // A ready report of the level drawn before it reveals nothing.
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE - 1 }),
    });
    expect(viewport.isRevealed).toBe(false);

    mockIpcCommands({
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [
        {
          group: ERenderTextureGroup.SURFACES,
          reference: "terrain\\terrain_escape",
          state: { height: 512, isExpanded: false, kind: "loaded", layout: "DXT1", levels: 10, width: 1024 },
        },
        { group: ERenderTextureGroup.SURFACES, reference: "act\\act_none", state: { kind: "missing" } },
      ],
      [ERenderCommand.MEASURE_SURFACES]: [{ drawables: 3, narrowest: SPAN, shaderId: 7, span: SPAN, triangles: 40 }],
    });
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({
        bytes: 20,
        isReady: true,
        scene: LEVEL_SCENE,
        sectors: 2,
        sectorsTotal: 2,
        textures: 3,
        texturesTotal: 3,
      }),
    });
    expect(viewport.isRevealed).toBe(true);

    // Counted once everything is resident, so it counts the level whole.
    await waitFor(() => expect(viewport.surfaceGeometry.get(7)?.triangles).toBe(40));
    expect(viewport.surfaceGeometry.get(7)).toEqual({ drawables: 3, narrowest: SPAN, span: SPAN, triangles: 40 });
    expect(viewport.textureReport.uploaded).toBe(2);
    expect(viewport.textureReport.dressing.get("terrain\\terrain_escape")?.upload).toBe("1024×512 · DXT1 · 10 levels");
    expect(viewport.textureReport.problems).toEqual([
      {
        group: ERenderTextureGroup.SURFACES,
        reason: "Nothing in the mounted roots answers to it",
        reference: "act\\act_none",
      },
    ]);
  });

  it("describes a resident level once, however its later reports change its counts", async () => {
    const { container } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    mockIpcCommands({
      [ERenderCommand.DESCRIBE_PROBLEMS]: { models: [], sectors: [], skipped: [] },
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [],
      [ERenderCommand.MEASURE_SURFACES]: [],
    });
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ bytes: 20, isReady: true, scene: LEVEL_SCENE }),
    });
    await waitFor(() => expect(viewport.description.isReady).toBe(true));

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ bytes: 30, isReady: true, scene: LEVEL_SCENE }),
    });
    await flush();

    expect(viewport.description.isReady).toBe(true);
    expect(listMockInvokeCalls(ERenderCommand.MEASURE_SURFACES)).toHaveLength(1);
  });

  // The backend moves the scene onto a level its session read again, so the page neither shows it again nor stands the
  // camera anew.
  it("reveals a level read again in its session once the scene it streams as is resident", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const load: LevelLoadService = container.get(LevelLoadService);

    jest.spyOn(service.log, "error").mockImplementation(() => undefined);
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });
    expect(viewport.isRevealed).toBe(true);

    const sessionId: string = load.level.value?.selected.sessionId as string;
    const resets: number = listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA).length;

    mockIpcCommands({
      [ELevelsCommand.RESTORE_LEVEL]: () => mockSessionSnapshot(mockSelectedLevelDescription(), sessionId, 1),
    });
    container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 1,
      root: { kind: "derived" },
      sessionId,
      staleParts: [],
      isViewRenewed: true,
    });
    await waitFor(() => expect(load.level.value?.selected.revision).toBe(1));

    expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toHaveLength(1);
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA)).toHaveLength(resets);
    expect(viewport.isRevealed).toBe(false);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });
    expect(viewport.isRevealed).toBe(false);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE + 1 }),
    });
    expect(viewport.isRevealed).toBe(true);
  });

  it("keeps a level whose map alone moved revealed in the scene it is shown in", async () => {
    const { container } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const load: LevelLoadService = container.get(LevelLoadService);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });

    const sessionId: string = load.level.value?.selected.sessionId as string;

    mockIpcCommands({
      [ELevelsCommand.DESCRIBE_LEVEL_MAP]: (args?: Record<string, unknown>) =>
        mockSessionSnapshot(null, args?.sessionId as string, 1),
    });
    container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 1,
      root: { kind: "kept" },
      sessionId,
      staleParts: [LEVEL_MAP_PART],
      isViewRenewed: false,
    });
    await waitFor(() => expect(load.levelMap.snapshot?.revision).toBe(1));

    expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toHaveLength(1);
    expect(viewport.isRevealed).toBe(true);
  });

  // The backend streams a level afresh on its geometry without reading the level itself again: the viewport waits for
  // the scene it streams as, the camera kept.
  it("waits for the scene a level streams as afresh on its geometry, reading the level itself nowhere again", async () => {
    const { container } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const load: LevelLoadService = container.get(LevelLoadService);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });

    const sessionId: string = load.level.value?.selected.sessionId as string;
    const resets: number = listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA).length;

    container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 1,
      root: { kind: "kept" },
      sessionId,
      staleParts: [LEVEL_GEOMETRY_PART],
      isViewRenewed: true,
    });

    expect(viewport.isRevealed).toBe(false);
    expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toHaveLength(1);
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA)).toHaveLength(resets);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE + 1 }),
    });

    expect(viewport.isRevealed).toBe(true);
  });

  it("reads frames and the camera into the readouts, and forgets the frame once detached", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    emitViewportEvent({ kind: ERenderViewportEvent.FRAME, report: REPORT });
    expect(service.frame).toBe(REPORT);

    emitViewportEvent({ kind: EWorldViewportEvent.CAMERA, pose: { position: [1, 2, 3], target: [1, 2, 2] } });
    // The readout states the engine's space, which mirrors renderer space along z.
    expect(viewport.camera?.position).toEqual({ x: 1, y: 2, z: -3 });

    service.detach();
    expect(service.frame.framesPerSecond).toBe(0);
  });

  it("names what a pick hit in the level's own coordinates", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const hit: RenderLevelHit = {
      isImpostor: false,
      kind: ERenderLevelHit.SURFACE,
      mesh: 2,
      place: 7,
      point: [1, 2, 3],
      sector: 4,
      shaderId: 11,
    };

    mockIpcCommands({ [ERenderCommand.PICK]: hit });
    await service.pick({ x: 10, y: 20 }, true);

    expect(listMockInvokeCalls(ERenderCommand.PICK).at(-1)).toEqual({
      viewport: LEVEL_VIEWPORT,
      request: { x: 10, y: 20, isThrough: true },
    });
    expect(container.get(LevelViewportService).picked).toEqual({
      isImpostor: false,
      kind: ELevelPick.SURFACE,
      mesh: 2,
      place: 7,
      point: { x: 1, y: 2, z: -3 },
      sector: 4,
      shaderId: 11,
    });
  });

  // The scene's keys act on it only while it has the keyboard, which leaving it or detaching takes away.
  it("notes whether the scene has the keyboard", async () => {
    const element: HTMLElement = document.createElement("div");
    const { container, service } = await mockAttachedLevelRender(element);
    const viewportService: LevelViewportService = container.get(LevelViewportService);

    element.dispatchEvent(new FocusEvent("focus"));
    expect(viewportService.isSceneFocused).toBe(true);

    element.dispatchEvent(new FocusEvent("blur"));
    expect(viewportService.isSceneFocused).toBe(false);

    element.dispatchEvent(new FocusEvent("focus"));
    service.detach();
    expect(viewportService.isSceneFocused).toBe(false);
  });

  // The renderer's clock starts at noon, so the time a level was left at reaches it with the level's weather.
  it("plays a level's weather on from the time it was last left at", async () => {
    writeLevelWeatherMemory(toLevelWeatherMemoryKey(mockSelectedLevelDescription()), {
      base: null,
      control: DEFAULT_LEVEL_WEATHER_CONTROL,
      cycle: "",
      manual: null,
      seed: null,
      source: ELevelWeatherSource.WEATHER,
      sunShafts: DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS,
      time: 7_200,
    });

    await mockAttachedLevelRender();

    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.SEEK_WEATHER)).toEqual([{ time: 7_200, viewport: LEVEL_VIEWPORT }])
    );
    expect(listMockInvokeCalls(ERenderCommand.PLAY_WEATHER).at(-1)?.play).toEqual({
      kind: EWorldWeatherPlay.CYCLE,
      name: "default_clear",
    });
  });

  // A refused command leaves the last frame on screen, so it is logged; once, since some are sent every frame.
  it("logs a command the viewport refused, once however often it is sent", async () => {
    const { service } = await mockAttachedLevelRender();
    const warn = jest.spyOn(service.log, "warn").mockImplementation(() => undefined);

    mockIpcCommands({ [ERenderCommand.SEEK_WEATHER]: () => Promise.reject(new Error("invalid time")) });
    service.seekWeather(60);
    service.seekWeather(120);

    await waitFor(() => expect(warn).toHaveBeenCalled());
    expect(warn).toHaveBeenCalledTimes(1);
    expect(warn).toHaveBeenCalledWith("A native viewport command failed:", "seekWeather", new Error("invalid time"));
  });

  it("states why the viewport cannot draw, and lets it go on detach", async () => {
    const { service } = await mockAttachedLevelRender();

    emitViewportEvent({ kind: ERenderViewportEvent.FAILURE, message: "No GPU" });
    expect(service.failure).toBe("No GPU");

    service.detach();

    await waitFor(() =>
      expect(listMockInvokeCalls(ERenderCommand.DETACH_VIEWPORT)).toEqual([{ viewport: LEVEL_VIEWPORT }])
    );
  });

  // A show the renderer refused would otherwise leave the cover spinning over nothing.
  it("states why the viewport could not show the level, until the next show", async () => {
    const { container, service } = await mockAttachedLevelRender();

    jest.spyOn(service.log, "error").mockImplementation(() => undefined);
    mockIpcCommands({ [ELevelsCommand.SHOW_LEVEL]: () => Promise.reject(new Error("The level is closed")) });
    await container.get(LevelLoadService).close();

    await waitFor(() => expect(service.failure).toBe("The level is closed"));

    mockIpcCommands({
      [ELevelsCommand.OPEN_LEVEL]: mockSessionResponse(mockSelectedLevelDescription()),
      [ELevelsCommand.SHOW_LEVEL]: LEVEL_SCENE,
    });
    await container.get(LevelLoadService).load({
      engine: EXrayEngineChoice.VANILLA,
      isDltx: false,
      roots: createRoots(["C:\\game"]),
      source: { kind: "asset", logicalPath: "levels\\zaton" },
    });

    expect(service.failure).toBeNull();
    await waitFor(() => expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toHaveLength(3));
  });

  it("holds why the resident level could not be described, for the panels to say", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    jest.spyOn(service.log, "error").mockImplementation(() => undefined);
    mockIpcCommands({
      [ERenderCommand.DESCRIBE_PROBLEMS]: () => Promise.reject(new Error("The viewport is gone")),
    });
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });

    expect(viewport.description.isLoading).toBe(true);
    await waitFor(() => expect(viewport.description.error?.message).toBe("The viewport is gone"));
  });
});
