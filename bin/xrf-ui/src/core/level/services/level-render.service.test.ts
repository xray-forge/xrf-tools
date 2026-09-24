import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";
import {
  EMPTY_RENDERER_STATIC_DRAW_REPORT,
  ERendererCameraController,
  ERendererOverlay,
  ERendererRequest,
  ERendererResponse,
  IRendererReport,
} from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";

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

function mockReport(position: [number, number, number]): IRendererReport {
  return {
    camera: { position, target: [position[0], position[1], position[2] - 1] },
    frame: { draws: 12, triangles: 400 } as IRendererReport["frame"],
    isGpuTimed: false,
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

  it("flies the camera from the level's start and frames it with the grid, the extent and the sun", async () => {
    const { service } = await mockAttached();

    expect(stub.take(ERendererRequest.CAMERA).at(-1)?.camera.kind).toBe(ERendererCameraController.FLY);
    expect(stub.take(ERendererRequest.PUT_OVERLAY).map((it) => it.key)).toEqual([
      "grid",
      "extent",
      "extent-box",
      "sun",
    ]);
    expect(stub.take(ERendererRequest.PUT_OVERLAY).at(-1)?.overlay.kind).toBe(ERendererOverlay.SUN);

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
    expect(stub.take(ERendererRequest.CONFIGURE).at(-1)?.settings.backdrop).toBe(0x202428);

    service.dispose();
  });

  it("gates the baked hemisphere by the baked light toggle", async () => {
    const { service, viewService } = await mockAttached();

    viewService.setOptions({ ...viewService.options, isLit: false });
    await stub.flush();

    expect(stub.take(ERendererRequest.CONFIGURE).at(-1)?.settings.hemiStrength).toBe(0);

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
