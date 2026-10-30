import { beforeEach, describe, expect, it, jest } from "@jest/globals";
import { waitFor } from "@testing-library/react";
import { EventBus, WireEvent } from "@wirestate/core";
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
import {
  LEVEL_GEOMETRY_PART,
  LEVEL_MAP_PART,
  LEVEL_SPAWN_PART,
  LEVEL_SURFACES_PART,
  LevelSpawnObject,
} from "@xrf/ipc/types/xrf-app-levels";
import { EXrayEngineChoice } from "@xrf/ipc/types/xrf-engine-target";
import { XraySurfaceDescriptor } from "@xrf/ipc/types/xrf-material";
import {
  ERenderAntialiasing,
  ERenderDlssAvailability,
  ERenderLevelHit,
  ERenderSelectionTarget,
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
  WorldSurfaceGeometry,
  WorldSurfaceSpan,
} from "@xrf/ipc/types/xrf-world";

import { createRoots } from "@/core/assets/lib";
import {
  emitViewportEvent,
  LEVEL_SCENE,
  LEVEL_VIEWPORT,
  mockAttachedLevelRender,
} from "@/core/level/fixtures/level-render";
import {
  mockLevelPartAnswer,
  mockLevelSpawnObject,
  mockSelectedLevelDescription,
} from "@/core/level/fixtures/level.mocks";
import { ELevelPick } from "@/core/level/lib/pick/level-pick";
import { ILevelResident } from "@/core/level/lib/render/level-resident";
import { DEFAULT_LEVEL_SUN_SHAFTS_OPTIONS } from "@/core/level/lib/weather/level-sun-shafts-options";
import { DEFAULT_LEVEL_WEATHER_CONTROL } from "@/core/level/lib/weather/level-weather-control";
import { toLevelWeatherMemoryKey, writeLevelWeatherMemory } from "@/core/level/lib/weather/level-weather-memory";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load";
import { LevelViewService } from "@/core/level/services/level-view";
import { LevelViewportService } from "@/core/level/services/level-viewport";
import { EMIT_NOTIFICATION_EVENT, ENotificationSeverity, INotificationPayload } from "@/core/notifications/lib";
import { mockRenderLoadReport } from "@/core/render/fixtures/render.mocks";
import { EMPTY_RENDER_FRAME_PHASES } from "@/core/render/lib/native/native-frame-report";
import { DEFAULT_RENDER_GRAPH_SETTINGS } from "@/core/render/lib/settings/render-graph-settings";
import { clearLocalStorage } from "@/core/storage";
import { mockSurfaceDescriptor, mockVisualTransform } from "@/core/visuals/fixtures/visual.mocks";
import { mockSessionResponse, mockSessionSnapshot } from "@/fixtures/mocks/session.mocks";
import { deferred, flush, IDeferred } from "@/fixtures/utils/promise";

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
    const { service } = await mockAttachedLevelRender();

    expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toEqual([
      { sessionId: expect.any(String), viewport: LEVEL_VIEWPORT },
    ]);
    expect(service.isRevealed).toBe(false);

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
    expect(service.isRevealed).toBe(false);

    // A ready report of the level drawn before it reveals nothing.
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE - 1 }),
    });
    expect(service.isRevealed).toBe(false);

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
    expect(service.isRevealed).toBe(true);

    // Counted once everything is resident, so it counts the level whole.
    await waitFor(() => expect(service.resident.isLoading).toBe(false));

    const resident: ILevelResident = service.resident.value as ILevelResident;

    expect(resident.surfaces.get(7)).toEqual({ drawables: 3, narrowest: SPAN, span: SPAN, triangles: 40 });
    expect(resident.textures.uploaded).toBe(2);
    expect(resident.textures.dressing.get("terrain\\terrain_escape")?.upload).toBe("1024×512 · DXT1 · 10 levels");
    expect(resident.textures.problems).toEqual([
      {
        group: ERenderTextureGroup.SURFACES,
        reason: "Nothing in the mounted roots answers to it",
        reference: "act\\act_none",
      },
    ]);
  });

  it("describes a resident level once, however its later reports change its counts", async () => {
    const { service } = await mockAttachedLevelRender();

    mockIpcCommands({
      [ERenderCommand.DESCRIBE_PROBLEMS]: { models: [], sectors: [], skipped: [] },
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [],
      [ERenderCommand.MEASURE_SURFACES]: [],
    });
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ bytes: 20, isReady: true, scene: LEVEL_SCENE }),
    });
    await waitFor(() => expect(service.resident.state.isReady).toBe(true));

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ bytes: 30, isReady: true, scene: LEVEL_SCENE }),
    });
    await flush();

    expect(service.resident.state.isReady).toBe(true);
    expect(listMockInvokeCalls(ERenderCommand.MEASURE_SURFACES)).toHaveLength(1);
  });

  // The backend moves the scene onto a level its session read again, so the page neither shows it again nor stands the
  // camera anew.
  it("reveals a level read again in its session once the scene it streams as is resident", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const load: LevelLoadService = container.get(LevelLoadService);

    jest.spyOn(service.log, "error").mockImplementation(() => undefined);
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });
    expect(service.isRevealed).toBe(true);

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
    expect(service.isRevealed).toBe(false);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });
    expect(service.isRevealed).toBe(false);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE + 1 }),
    });
    expect(service.isRevealed).toBe(true);
  });

  it("keeps a level whose map alone moved revealed in the scene it is shown in", async () => {
    const { container, service } = await mockAttachedLevelRender();
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
    expect(service.isRevealed).toBe(true);
  });

  // The backend streams a level afresh on its geometry without reading the level itself again: the viewport waits for
  // the scene it streams as, the camera kept.
  it("waits for the scene a level streams as afresh on its geometry, reading the level itself nowhere again", async () => {
    const { container, service } = await mockAttachedLevelRender();
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

    expect(service.isRevealed).toBe(false);
    expect(listMockInvokeCalls(ELevelsCommand.SHOW_LEVEL)).toHaveLength(1);
    expect(listMockInvokeCalls(ERenderCommand.COMMAND_CAMERA)).toHaveLength(resets);

    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE + 1 }),
    });

    expect(service.isRevealed).toBe(true);
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
      depth: 3,
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

  it("focuses the view's lens where a click lands while the focus is picked, selecting nothing", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);

    mockIpcCommands({
      [ERenderCommand.PICK]: { depth: 12.345, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] },
    });
    viewport.setFocusPicking(true);
    await service.focusAt({ x: 10, y: 20 }, false);

    expect(viewport.isFocusPicking).toBe(false);
    expect(viewport.picked).toBeNull();
    expect(container.get(LevelViewService).features.camera).toEqual({ focusDistance: 12.35, isDefocused: true });
  });

  it("keeps a spawned object picked across its spawn read anew, found by its name, and lets it go once gone", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const load: LevelLoadService = container.get(LevelLoadService);
    const sessionId: string = load.level.value?.selected.sessionId as string;
    const lamp = mockLevelSpawnObject({ index: 0, name: "lamp" });
    const crate = mockLevelSpawnObject({
      index: 1,
      name: "crate",
      transform: mockVisualTransform({ x: 1, y: 2, z: 3 }),
    });

    async function relist(revision: number, objects: Array<LevelSpawnObject>): Promise<void> {
      mockIpcCommands({
        [ELevelsCommand.DESCRIBE_SPAWN_OBJECTS]: mockLevelPartAnswer({ objects, visuals: ["lamp"] }, revision),
        [ERenderCommand.PICK]: { depth: 0, kind: ERenderLevelHit.SPAWN, object: 1, point: [0, 0, 0] },
      });
      container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
        revision,
        root: { kind: "kept" },
        sessionId,
        staleParts: [LEVEL_SPAWN_PART],
        isViewRenewed: false,
      });
      await waitFor(() => expect(load.spawnRevision).toBe(revision));
    }

    await relist(1, [lamp, crate]);
    await service.pick({ x: 10, y: 20 }, false);
    expect(viewport.picked).toMatchObject({ kind: ELevelPick.SPAWN, object: crate });

    await relist(2, [
      { ...crate, index: 0 },
      { ...lamp, index: 1 },
    ]);
    expect(viewport.picked).toMatchObject({ kind: ELevelPick.SPAWN, object: { index: 0, name: "crate" } });
    expect(viewport.isPickRelocated).toBe(true);
    expect(listMockInvokeCalls(ERenderCommand.SET_SELECTION).at(-1)?.selection?.target).toEqual({
      kind: ERenderSelectionTarget.SPAWN,
      object: 0,
    });

    await relist(3, [lamp]);
    expect(viewport.picked).toBeNull();
  });

  // A table read anew may number its entries afresh; the surface keeps the place it was picked at.
  it("keeps a surface picked across its level streamed afresh, its entry found again by shader and textures", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const viewport: LevelViewportService = container.get(LevelViewportService);
    const load: LevelLoadService = container.get(LevelLoadService);
    const sessionId: string = load.level.value?.selected.sessionId as string;
    const wall = mockSurfaceDescriptor({ shader: "default", textures: ["wall"] });
    const floor = mockSurfaceDescriptor({ shader: "default", textures: ["floor"] });

    async function reread(revision: number, surfaces: Array<XraySurfaceDescriptor>): Promise<void> {
      mockIpcCommands({
        [ELevelsCommand.DESCRIBE_SURFACES]: mockLevelPartAnswer(surfaces, revision),
        [ERenderCommand.PICK]: {
          depth: 0,
          isImpostor: false,
          kind: ERenderLevelHit.SURFACE,
          mesh: null,
          place: null,
          point: [0, 0, 0],
          sector: 4,
          shaderId: 0,
        },
      });
      container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
        revision,
        root: { kind: "kept" },
        sessionId,
        staleParts: [LEVEL_GEOMETRY_PART, LEVEL_SURFACES_PART],
        isViewRenewed: true,
      });
      await waitFor(() => expect(load.surfaces.snapshot?.revision).toBe(revision));
    }

    await reread(1, [wall, floor]);
    await service.pick({ x: 10, y: 20 }, false);
    await reread(2, [floor, wall]);

    expect(viewport.picked).toMatchObject({ kind: ELevelPick.SURFACE, sector: 4, shaderId: 1 });

    await reread(3, [floor]);

    expect(viewport.picked).toBeNull();
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

  it("says why the resident level could not be described, and holds it for the panels to say", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const raised: Array<INotificationPayload> = [];

    container
      .get(EventBus)
      .subscribe(EMIT_NOTIFICATION_EVENT, (event: WireEvent<INotificationPayload>) =>
        raised.push(event.payload as INotificationPayload)
      );
    jest.spyOn(service.log, "error").mockImplementation(() => undefined);
    mockIpcCommands({
      [ERenderCommand.DESCRIBE_PROBLEMS]: () => Promise.reject(new Error("The viewport is gone")),
    });
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });

    expect(service.resident.isLoading).toBe(true);
    await waitFor(() => expect(service.resident.error?.message).toBe("The viewport is gone"));
    expect(raised).toEqual([
      expect.objectContaining({
        details: "The viewport is gone",
        severity: ENotificationSeverity.ERROR,
        title: "Could not describe the resident level",
      }),
    ]);
  });

  // What was asked of a scene names that scene alone: its answer lands on no level opened or streamed since.
  it("drops a description and a pick asked of a scene let go before they answered", async () => {
    const { container, service } = await mockAttachedLevelRender();
    const load: LevelLoadService = container.get(LevelLoadService);
    const measured: IDeferred<Array<WorldSurfaceGeometry>> = deferred();
    const picked: IDeferred<RenderLevelHit> = deferred();

    mockIpcCommands({
      [ERenderCommand.DESCRIBE_PROBLEMS]: { models: [], sectors: [], skipped: [] },
      [ERenderCommand.DESCRIBE_TEXTURE_LOADS]: [],
      [ERenderCommand.MEASURE_SURFACES]: () => measured.promise,
      [ERenderCommand.PICK]: () => picked.promise,
    });
    emitViewportEvent({
      kind: ERenderViewportEvent.LOAD,
      report: mockRenderLoadReport({ isReady: true, scene: LEVEL_SCENE }),
    });
    void service.pick({ x: 10, y: 20 }, false);
    expect(service.resident.isLoading).toBe(true);

    container.get(EventBus).emit<SessionChange>(SESSION_CHANGE_EVENT, {
      revision: 1,
      root: { kind: "kept" },
      sessionId: load.level.value?.selected.sessionId as string,
      staleParts: [LEVEL_GEOMETRY_PART],
      isViewRenewed: true,
    });
    expect(service.isRevealed).toBe(false);
    expect(service.resident.state.isIdle).toBe(true);

    measured.resolve([{ drawables: 3, narrowest: SPAN, shaderId: 7, span: SPAN, triangles: 40 }]);
    picked.resolve({
      depth: 0,
      isImpostor: false,
      kind: ERenderLevelHit.SURFACE,
      mesh: null,
      place: null,
      point: [0, 0, 0],
      sector: 4,
      shaderId: 0,
    });
    await flush();

    expect(service.resident.value).toBeNull();
    expect(container.get(LevelViewportService).picked).toBeNull();
  });
});
