import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";
import {
  DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDERER_SHADOW_SETTINGS,
  EMPTY_RENDERER_LIGHTS_REPORT,
  EMPTY_RENDERER_STATIC_DRAW_REPORT,
  ERendererAntialiasing,
  ERendererCameraCommand,
  ERendererCameraController,
  ERendererOverlay,
  ERendererRequest,
  ERendererResponse,
  ERendererTextureEncoding,
  ERendererWeatherTransition,
  IRendererReport,
  IRendererSettings,
  TRendererRequest,
} from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";
import { Maybe } from "@xrf/types";

import { IPC_METRICS } from "@/core/ipc/metrics";
import { ELevelSpawnCategory } from "@/core/ipc/types/xrf-app";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { ELevelSurfaceDressing } from "@/core/level/lib/surface/level-surface-dressing";
import { ELevelWeatherSource } from "@/core/level/lib/weather/level-weather-source";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { LevelWeatherService } from "@/core/level/services/level-weather.service";
import { setMockBulkResponses } from "@/fixtures/mocks/bulk.mocks";
import {
  mockLevelSpawnModel,
  mockLevelSpawnObject,
  mockLevelTextureReference,
  mockSelectedLevelDescription,
} from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { InvokeMap, resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockLevelWeatherDescription } from "@/fixtures/mocks/weather.mocks";
import { mockContainer } from "@/fixtures/utils/container";
import { mockRendererThread } from "@/fixtures/utils/renderer";

let stub: IRendererWorkerStub;
let LevelRenderService: typeof import("./level-render.service").LevelRenderService;

async function mockAttached(responses: InvokeMap = {}): Promise<{
  container: Container;
  service: InstanceType<typeof LevelRenderService>;
  viewService: LevelViewService;
}> {
  setMockInvokeResponses({
    ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()),
    ...responses,
  });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelViewService,
    LevelViewportService,
    LevelRenderService,
    LevelWeatherService,
  ]);

  await container.get(LevelLoadService).restore();

  const service = container.get(LevelRenderService);

  service.attach(document.createElement("div"));
  await stub.flush();

  return { container, service, viewService: container.get(LevelViewService) };
}

function drawnSettings(): Maybe<IRendererSettings> {
  return stub.take(ERendererRequest.CONFIGURE).at(-1)?.settings ?? stub.take(ERendererRequest.START).at(-1)?.settings;
}

function mockReport(position: [number, number, number]): IRendererReport {
  return {
    camera: { position, target: [position[0], position[1], position[2] - 1] },
    cpuMemory: 0,
    frame: { draws: 12, triangles: 400 } as IRendererReport["frame"],
    isGpuTimed: false,
    lights: EMPTY_RENDERER_LIGHTS_REPORT,
    passes: [],
    staticDraws: EMPTY_RENDERER_STATIC_DRAW_REPORT,
    weather: null,
  };
}

async function takeSettle(): Promise<Extract<TRendererRequest, { kind: ERendererRequest.SETTLE }>> {
  for (let flush: number = 0; flush < 20 && !stub.take(ERendererRequest.SETTLE).length; flush += 1) {
    await stub.flush();
  }

  const [settle] = stub.take(ERendererRequest.SETTLE);

  expect(settle).toBeDefined();

  return settle;
}

beforeAll(async () => {
  mockRendererThread(() => stub.worker);

  ({ LevelRenderService } = await import("./level-render.service"));
});

beforeEach(() => {
  stub = createRendererWorkerStub();
  resetMockInvoke();
  window.localStorage.clear();
});

describe("LevelRenderService", () => {
  it("starts nothing until a view attaches", () => {
    mockContainer([
      LevelLoadService,
      LevelViewService,
      LevelViewportService,
      LevelRenderService,
      LevelWeatherService,
    ]).get(LevelRenderService);

    expect(stub.requests).toHaveLength(0);
  });

  // The grid and the extent it frames are off until the toolbar asks for them: a level reads as itself first.
  it("flies the camera from the level's start with the sun, and frames it with the grid and extent on asking", async () => {
    const { service, viewService } = await mockAttached();

    expect(stub.take(ERendererRequest.CAMERA).at(-1)?.camera.kind).toBe(ERendererCameraController.FLY);
    expect(stub.take(ERendererRequest.PUT_OVERLAY).map((it) => it.key)).toEqual(["sun"]);
    expect(stub.take(ERendererRequest.PUT_OVERLAY).at(-1)?.overlay.kind).toBe(ERendererOverlay.SUN);

    viewService.setOptions({ ...viewService.options, isGridVisible: true });
    await stub.flush();

    expect(stub.take(ERendererRequest.PUT_OVERLAY).map((it) => it.key)).toEqual(
      expect.arrayContaining(["grid", "extent", "extent-box"])
    );

    service.dispose();
  });

  it("plays the level's weather from where the clock stands, told the view's toggles and every seek", async () => {
    const { container, service, viewService } = await mockAttached({
      ["plugin:levels|read_level_weather"]: mockSessionResponse(mockLevelWeatherDescription()),
      ["plugin:levels|resolve_level_textures"]: mockSessionResponse(({ references }: { references: Array<string> }) =>
        references.map((reference: string) => mockLevelTextureReference(reference))
      ),
    });
    const weatherService: LevelWeatherService = container.get(LevelWeatherService);

    for (let flush: number = 0; flush < 20 && !weatherService.weather; flush += 1) {
      await stub.flush();
    }

    await stub.flush();

    expect(stub.take(ERendererRequest.WEATHER).at(-1)?.weather?.keyframes).toHaveLength(2);
    expect(stub.take(ERendererRequest.WEATHER).at(-1)?.transition).toBe(ERendererWeatherTransition.CUT);
    expect(stub.take(ERendererRequest.WEATHER_CONTROL).find((it) => it.control.time !== null)?.control).toEqual({
      factor: 12,
      isClouded: true,
      isDynamicSun: false,
      isFogged: true,
      isPaused: true,
      isRainy: true,
      isThundering: true,
      isWindy: true,
      time: 43_200,
    });

    viewService.setOptions({ ...viewService.options, isFogged: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.WEATHER_CONTROL).at(-1)?.control).toMatchObject({ isFogged: false, time: null });

    weatherService.seekTo(3_600);
    await stub.flush();

    expect(stub.take(ERendererRequest.WEATHER_CONTROL).at(-1)?.control.time).toBe(3_600);

    weatherService.playEffect("fx_blowout");
    await stub.flush();

    expect(stub.take(ERendererRequest.WEATHER_EFFECT).at(-1)?.effect).toBe("fx_blowout");

    weatherService.setSource(ELevelWeatherSource.MANUAL);

    for (let flush: number = 0; flush < 20 && weatherService.weather !== weatherService.manualPlayable; flush += 1) {
      await stub.flush();
    }

    await stub.flush();

    // The keyframe set by hand is faded into, its sun standing by its own angles.
    expect(stub.take(ERendererRequest.WEATHER).at(-1)?.weather?.keyframes).toHaveLength(1);
    expect(stub.take(ERendererRequest.WEATHER).at(-1)?.transition).toBe(ERendererWeatherTransition.FADE);
    expect(stub.take(ERendererRequest.WEATHER_CONTROL).at(-1)?.control.isDynamicSun).toBe(false);

    service.dispose();
  });

  it("draws impostors while the toolbar asks, at the distance it sets", async () => {
    const { service, viewService } = await mockAttached();

    expect(drawnSettings()?.features.lod.isImpostors).toBe(true);

    viewService.setLod({ distance: 2 });
    await stub.flush();

    expect(drawnSettings()?.features.lod.geometryLod).toBeCloseTo(3);

    viewService.setOptions({ ...viewService.options, isImpostors: false });
    await stub.flush();

    expect(drawnSettings()?.features.lod.isImpostors).toBe(false);

    service.dispose();
  });

  it("draws shadows and smooths edges as the toolbar sets them over the settings", async () => {
    const { service, viewService } = await mockAttached();

    function features() {
      return drawnSettings()?.features;
    }

    expect(features()?.shadows.isEnabled).toBe(true);
    expect(features()?.antialiasing).toBe(ERendererAntialiasing.SMAA);

    viewService.setFeatures({
      ambientOcclusion: { radius: 2 },
      antialiasing: ERendererAntialiasing.FXAA,
      grass: {},
      lights: {},
      shadows: { cascades: [20], filter: 0 },
      water: {},
    });
    await stub.flush();

    const set = features();

    expect(set?.antialiasing).toBe(ERendererAntialiasing.FXAA);
    expect(set?.shadows.cascades).toEqual([20]);
    expect(set?.shadows.filter).toBe(0);
    expect(set?.shadows.resolution).toBe(DEFAULT_RENDERER_SHADOW_SETTINGS.resolution);
    expect(set?.ambientOcclusion).toEqual({ ...DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS, radius: 2 });

    viewService.setOptions({ ...viewService.options, isAntialiased: false, isOccluded: false, isShadowed: false });
    await stub.flush();

    const off = features();

    expect(off?.antialiasing).toBe(ERendererAntialiasing.NONE);
    expect(off?.shadows.isEnabled).toBe(false);
    expect(off?.ambientOcclusion.isEnabled).toBe(false);

    service.dispose();
  });

  it("gates the baked hemisphere by the baked light toggle", async () => {
    const { service, viewService } = await mockAttached();

    viewService.setOptions({ ...viewService.options, isBaked: false });
    await stub.flush();

    expect(drawnSettings()?.hemiStrength).toBe(0);

    service.dispose();
  });

  // Textures off is a uniform the surfaces' shaders read: turning them back on fetches and uploads nothing again.
  it("draws the surfaces' textures by the textures toggle, putting no surface or texture again", async () => {
    const { service, viewService } = await mockAttached();
    const surfaces: number = stub.take(ERendererRequest.PUT_SURFACE).length;
    const textures: number = stub.take(ERendererRequest.PUT_TEXTURE).length;

    expect(drawnSettings()?.isTextured).toBe(true);

    viewService.setOptions({ ...viewService.options, isTextured: false });
    await stub.flush();

    expect(drawnSettings()?.isTextured).toBe(false);

    viewService.setOptions({ ...viewService.options, isTextured: true });
    await stub.flush();

    expect(drawnSettings()?.isTextured).toBe(true);
    expect(stub.take(ERendererRequest.PUT_SURFACE)).toHaveLength(surfaces);
    expect(stub.take(ERendererRequest.PUT_TEXTURE)).toHaveLength(textures);
    expect(stub.take(ERendererRequest.RELEASE_SURFACE)).toHaveLength(0);

    service.dispose();
  });

  // A renderer is told its settings as it starts, and then only what changed: a configure rebuilds passes, and a
  // toggle the settings do not read has nothing to rebuild. The weather lights the level, never a lighting of its own.
  it("configures only for what changed", async () => {
    const { service, viewService } = await mockAttached();

    expect(stub.take(ERendererRequest.START)).toHaveLength(1);
    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(0);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(0);

    viewService.setOptions({ ...viewService.options, isAxesVisible: true, isGridVisible: true, isSunVisible: false });
    viewService.setOptions({ ...viewService.options, isStatsVisible: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(0);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(0);

    viewService.setOptions({ ...viewService.options, isFogged: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(0);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(0);

    viewService.setOptions({ ...viewService.options, isShadowed: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(1);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(0);

    service.dispose();
  });

  // The same start describes the same camera, which the renderer keeps flown where it was: opening it again stood
  // nothing back at it.
  it("stands the camera back at the start when the same level opens again", async () => {
    const { container, service } = await mockAttached();
    const loadService: LevelLoadService = container.get(LevelLoadService);

    loadService.clear();
    await loadService.restore();
    await stub.flush();

    const kinds: Array<ERendererRequest> = stub.requests.map((it: TRendererRequest) => it.kind);
    const stood: number = kinds.lastIndexOf(ERendererRequest.CAMERA);
    const reset: TRendererRequest = stub.requests[kinds.indexOf(ERendererRequest.CAMERA_COMMAND, stood)];

    expect(reset).toMatchObject({ command: { kind: ERendererCameraCommand.RESET } });

    service.dispose();
  });

  // New speeds or a new lens are not a request to go back to the start.
  it("keeps the camera's start when the toolbar changes how it flies", async () => {
    const { service, viewService } = await mockAttached();
    const [first] = stub.take(ERendererRequest.CAMERA);

    viewService.setCamera({ ...viewService.camera, speed: 42 });
    await stub.flush();

    const last = stub.take(ERendererRequest.CAMERA).at(-1);

    expect(last?.camera).toMatchObject({ position: first.camera.position, speed: 42, target: first.camera.target });

    service.dispose();
  });

  it("says what the frames cost and where the camera is, in the level's own axes", async () => {
    const { container, service } = await mockAttached();

    stub.respond({ kind: ERendererResponse.REPORT, report: mockReport([1, 2, 3]) });

    const viewport: LevelViewportService = container.get(LevelViewportService);

    expect(viewport.stats.draws).toBe(12);
    expect(viewport.camera?.position).toEqual({ x: 1, y: 2, z: -3 });

    service.dispose();
  });

  it("hands the renderer where to fetch a level's textures, and says what each came to once it is told", async () => {
    const { container, service } = await mockAttached({
      ["plugin:levels|open_lights"]: mockSessionResponse({
        lights: { animators: [], lights: [] },
        projectors: [mockLevelTextureReference("lamp")],
      }),
    });
    const viewport: LevelViewportService = container.get(LevelViewportService);

    await container.get(LevelLoadService).whenHeldRead();
    await stub.flush();

    const put = stub.take(ERendererRequest.PUT_TEXTURE).find((it) => it.key === "lamp");

    expect(put?.source.encoding).toBe(ERendererTextureEncoding.FETCH);
    expect(viewport.textureReport.dressing.get("lamp")?.state).toBe(ELevelSurfaceDressing.FETCHING);

    stub.respond({
      fetch: { bytes: 2048, duration: 5, failure: null, isDecoded: false, size: { height: 4, levels: 1, width: 4 } },
      key: "lamp",
      kind: ERendererResponse.TEXTURE_FETCHED,
    });

    expect(viewport.textureReport.dressing.get("lamp")).toMatchObject({
      state: ELevelSurfaceDressing.UPLOADED,
      upload: "4×4 · 1 level",
    });
    // Counted beside the page's own fetches, since the bytes crossed no seam the page measures.
    expect(IPC_METRICS.read().commands.find((it) => it.command === "renderer|fetch_texture")?.received).toBe(2048);

    service.dispose();
  });

  it("asks the loader to stream only once the camera has gone far enough", async () => {
    const { container, service } = await mockAttached();
    const loadService = container.get(LevelLoadService) as unknown as { stream: (point: ILevelPoint) => Promise<void> };
    const stream = jest.spyOn(loadService, "stream");

    // Well away from where the level opened, which streamed as it opened.
    stub.respond({ kind: ERendererResponse.REPORT, report: mockReport([0, 0, 100]) });
    stub.respond({ kind: ERendererResponse.REPORT, report: mockReport([0, 0, 101]) });
    stub.respond({ kind: ERendererResponse.REPORT, report: mockReport([0, 0, 150]) });

    expect(stream.mock.calls.map(([point]) => point.z)).toEqual([100, 150]);

    service.dispose();
  });

  // A level assembling in view is worse than a wait: shown only once the renderer says a frame was drawn after
  // everything the level opens with was handed to it.
  it("keeps the level covered until the renderer has drawn it with everything it opens with", async () => {
    const { container, service } = await mockAttached();
    const viewportService: LevelViewportService = container.get(LevelViewportService);
    const settle = await takeSettle();

    expect(viewportService.isRevealed).toBe(false);

    stub.respond({ id: settle.id, kind: ERendererResponse.SETTLED });
    await stub.flush();

    expect(viewportService.isRevealed).toBe(true);

    service.dispose();
  });

  it("stands the camera where it is sent, anew, and reads the level around it", async () => {
    const { container, service } = await mockAttached();
    const loadService = container.get(LevelLoadService) as unknown as { stream: (point: ILevelPoint) => Promise<void> };
    const stream = jest.spyOn(loadService, "stream");

    service.goTo({ heading: 0, pitch: 0, x: 10, y: 2, z: 300 });
    await stub.flush();

    // Stated in the level's own axes, stood in the renderer's.
    expect(stub.take(ERendererRequest.CAMERA).at(-1)?.camera.position).toEqual([10, 2, -300]);
    expect(stub.take(ERendererRequest.CAMERA_COMMAND).at(-1)?.command.kind).toBe(ERendererCameraCommand.RESET);
    expect(stream).toHaveBeenLastCalledWith({ x: 10, y: 2, z: -300 });

    service.dispose();
  });

  // The loader resolves what a closed level was streaming, which used to reveal the viewport over nothing.
  it("reveals nothing for a level closed while it was prepared", async () => {
    const { container, service } = await mockAttached();
    const settle = await takeSettle();

    container.get(LevelLoadService).clear();
    stub.respond({ id: settle.id, kind: ERendererResponse.SETTLED });
    await stub.flush();

    expect(container.get(LevelViewportService).isRevealed).toBe(false);

    service.dispose();
  });

  it("keeps the level covered and says why when the renderer fails", async () => {
    const { container, service } = await mockAttached();

    await takeSettle();
    stub.respond({ kind: ERendererResponse.FAILED, reason: "No WebGPU adapter" });
    await stub.flush();

    expect(service.failure).toBe("No WebGPU adapter");
    expect(container.get(LevelViewportService).isRevealed).toBe(false);

    service.dispose();
  });

  it("opens the level again in the renderer a view shown after a failure starts", async () => {
    const { container, service } = await mockAttached();

    await takeSettle();
    stub.respond({ kind: ERendererResponse.FAILED, reason: "Device lost" });
    service.detach();

    stub = createRendererWorkerStub();
    service.attach(document.createElement("div"));

    const settle = await takeSettle();

    expect(service.failure).toBeNull();
    expect(stub.take(ERendererRequest.START)).toHaveLength(1);

    stub.respond({ id: settle.id, kind: ERendererResponse.SETTLED });
    await stub.flush();

    expect(container.get(LevelViewportService).isRevealed).toBe(true);

    service.dispose();
  });

  // A renderer started after the spawn was read is handed it at once, and a category hidden by then is never put.
  it("stands only the shown categories of a spawn held before the renderer started", async () => {
    setMockInvokeResponses({
      ["plugin:levels|describe_spawn_models"]: mockSessionResponse({
        failures: [],
        hemi: [],
        models: [mockLevelSpawnModel("crate").description],
      }),
      ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()),
      ["plugin:levels|open_spawn_objects"]: mockSessionResponse({
        objects: [
          mockLevelSpawnObject({ index: 0 }),
          mockLevelSpawnObject({ category: ELevelSpawnCategory.ITEMS, index: 1 }),
        ],
        visuals: ["crate"],
      }),
    });
    setMockBulkResponses({ "levels/read_spawn_model": mockLevelSpawnModel("crate").buffer });

    const container: Container = mockContainer([
      LevelLoadService,
      LevelViewService,
      LevelViewportService,
      LevelRenderService,
      LevelWeatherService,
    ]);
    const viewService: LevelViewService = container.get(LevelViewService);

    viewService.setOptions({ ...viewService.options, isSpawnedProps: false });
    await container.get(LevelLoadService).restore();
    await container.get(LevelLoadService).whenHeldRead();

    const service = container.get(LevelRenderService);

    service.attach(document.createElement("div"));
    await stub.flush();

    const put: Array<string> = stub.take(ERendererRequest.PUT_OBJECT).map((it) => it.key);

    expect(put).toContain(LEVEL_RENDER_KEYS.spawnObject(0, ELevelSpawnCategory.ITEMS));
    expect(put).not.toContain(LEVEL_RENDER_KEYS.spawnObject(0, ELevelSpawnCategory.PROPS));
    expect(stub.take(ERendererRequest.RELEASE_OBJECT)).toEqual([]);

    service.dispose();
  });

  it("keeps the renderer when the view goes, and lets it go on deactivation", async () => {
    const { service } = await mockAttached();

    service.detach();
    await stub.flush();

    expect(stub.requests.at(-1)?.kind).toBe(ERendererRequest.DETACH_VIEW);
    expect(stub.isTerminated()).toBe(false);

    service.dispose();

    expect(stub.isTerminated()).toBe(true);
  });
});
