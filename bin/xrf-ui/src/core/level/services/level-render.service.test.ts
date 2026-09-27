import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";
import {
  DEFAULT_RENDERER_AMBIENT_OCCLUSION_SETTINGS,
  DEFAULT_RENDERER_SHADOW_SETTINGS,
  DEFAULT_RENDERER_TREE_WIND,
  EMPTY_RENDERER_LIGHTS_REPORT,
  EMPTY_RENDERER_STATIC_DRAW_REPORT,
  ERendererAntialiasing,
  ERendererCameraController,
  ERendererOverlay,
  ERendererRequest,
  ERendererResponse,
  IRendererReport,
  IRendererSettings,
} from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";
import { Maybe } from "@xrf/types";

import { DEFAULT_LEVEL_FOG, toLevelRendererFog } from "@/core/level/lib/lighting/level-fog";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { LevelLoadService } from "@/core/level/services/level-load.service";
import { LevelViewService } from "@/core/level/services/level-view.service";
import { LevelViewportService } from "@/core/level/services/level-viewport.service";
import { mockSelectedLevelDescription } from "@/fixtures/mocks/level.mocks";
import { mockSessionResponse } from "@/fixtures/mocks/session.mocks";
import { resetMockInvoke, setMockInvokeResponses } from "@/fixtures/mocks/tauri.mocks";
import { mockContainer } from "@/fixtures/utils/container";

let stub: IRendererWorkerStub;
let LevelRenderService: typeof import("./level-render.service").LevelRenderService;

beforeAll(async () => {
  // The worker entry reads `import.meta.url`, which the test transform cannot, and the thread is what is stubbed.
  jest.doMock("@xrf/renderer/worker", () => ({ createRendererWorker: () => stub.worker }));
  HTMLCanvasElement.prototype.transferControlToOffscreen = function () {
    return {} as OffscreenCanvas;
  };

  ({ LevelRenderService } = await import("./level-render.service"));
});

beforeEach(() => {
  stub = createRendererWorkerStub();
  resetMockInvoke();
  window.localStorage.clear();
});

async function mockAttached(): Promise<{
  container: Container;
  service: InstanceType<typeof LevelRenderService>;
  viewService: LevelViewService;
}> {
  setMockInvokeResponses({ ["plugin:levels|get_level"]: mockSessionResponse(mockSelectedLevelDescription()) });

  const container: Container = mockContainer([
    LevelLoadService,
    LevelViewService,
    LevelViewportService,
    LevelRenderService,
  ]);

  await container.get(LevelLoadService).restore();

  const service = container.get(LevelRenderService);

  service.attach(document.createElement("div"));
  await stub.flush();

  return { container, service, viewService: container.get(LevelViewService) };
}

/** The settings the renderer draws with now: the last it was configured with, or those it started with. */
function drawnSettings(): Maybe<IRendererSettings> {
  return stub.take(ERendererRequest.CONFIGURE).at(-1)?.settings ?? stub.take(ERendererRequest.START).at(-1)?.settings;
}

function mockReport(position: [number, number, number]): IRendererReport {
  return {
    camera: { position, target: [position[0], position[1], position[2] - 1] },
    frame: { draws: 12, triangles: 400 } as IRendererReport["frame"],
    isGpuTimed: false,
    lights: EMPTY_RENDERER_LIGHTS_REPORT,
    passes: [],
    staticDraws: EMPTY_RENDERER_STATIC_DRAW_REPORT,
  };
}

describe("LevelRenderService", () => {
  it("starts nothing until a view attaches", () => {
    mockContainer([LevelLoadService, LevelViewService, LevelViewportService, LevelRenderService]).get(
      LevelRenderService
    );

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

  // The game's noon closes the level in at 350 metres; off, a whole level is inspectable from anywhere in it.
  it("draws the fog while the toolbar asks, as its controls set it", async () => {
    const { service, viewService } = await mockAttached();

    expect(stub.take(ERendererRequest.LIGHTING).at(-1)?.lighting.fog).toEqual(toLevelRendererFog(DEFAULT_LEVEL_FOG));

    viewService.setLighting({ ...viewService.lighting, fogDistance: 800 });
    await stub.flush();

    expect(stub.take(ERendererRequest.LIGHTING).at(-1)?.lighting.fog?.distance).toBe(800);

    viewService.setOptions({ ...viewService.options, isFogged: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.LIGHTING).at(-1)?.lighting.fog).toBeNull();
    expect(drawnSettings()?.backdrop).toBe(0x202428);

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

  // The game sways its trees by the weather's wind; off, they stand still for an inspection.
  it("sways the trees while the toolbar asks, as its controls set the wind", async () => {
    const { service, viewService } = await mockAttached();

    expect(stub.take(ERendererRequest.LIGHTING).at(-1)?.lighting.trees).toEqual(DEFAULT_RENDERER_TREE_WIND);

    viewService.setLighting({ ...viewService.lighting, windAmplitude: 0.02 });
    await stub.flush();

    expect(stub.take(ERendererRequest.LIGHTING).at(-1)?.lighting.trees?.amplitude).toBe(0.02);

    viewService.setOptions({ ...viewService.options, isWindy: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.LIGHTING).at(-1)?.lighting.trees).toBeNull();

    service.dispose();
  });

  it("gates the baked hemisphere by the baked light toggle", async () => {
    const { service, viewService } = await mockAttached();

    viewService.setOptions({ ...viewService.options, isBaked: false });
    await stub.flush();

    expect(drawnSettings()?.hemiStrength).toBe(0);

    service.dispose();
  });

  // A renderer is told its settings and its light as it starts, and then only what changed: a configure rebuilds
  // passes, and a toggle the settings do not read has nothing to rebuild.
  it("configures and lights only for what changed", async () => {
    const { service, viewService } = await mockAttached();

    expect(stub.take(ERendererRequest.START)).toHaveLength(1);
    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(0);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(1);

    viewService.setOptions({ ...viewService.options, isAxesVisible: true, isGridVisible: true, isSunVisible: false });
    viewService.setOptions({ ...viewService.options, isStatsVisible: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(0);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(1);

    viewService.setOptions({ ...viewService.options, isFogged: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(0);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(2);

    viewService.setOptions({ ...viewService.options, isShadowed: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE)).toHaveLength(1);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(2);

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

    for (let flush: number = 0; flush < 20 && !stub.take(ERendererRequest.SETTLE).length; flush += 1) {
      await stub.flush();
    }

    const [settle] = stub.take(ERendererRequest.SETTLE);

    expect(settle).toBeDefined();
    expect(viewportService.isRevealed).toBe(false);

    stub.respond({ id: settle.id, kind: ERendererResponse.SETTLED });
    await stub.flush();

    expect(viewportService.isRevealed).toBe(true);

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
