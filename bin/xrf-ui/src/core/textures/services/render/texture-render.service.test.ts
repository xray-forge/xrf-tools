import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";
import { runInAction } from "@wirestate/mobx";
import {
  EMPTY_RENDER_FRAME_COST,
  ERendererBumpPlane,
  ERendererCaptureSource,
  ERendererDraw,
  ERendererRequest,
  ERendererResponse,
  ERendererTextureEncoding,
  IRendererReport,
} from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";

import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/texture-lighting";
import {
  EMPTY_TEXTURE_SURFACE,
  ETextureSurfaceAlpha,
  ITextureSurfaceFile,
  ITextureSurfaceFiles,
} from "@/core/textures/lib/texture-surface";
import { TextureSurfaceService } from "@/core/textures/services/surface";
import { TextureViewService } from "@/core/textures/services/view";
import { mockContainer } from "@/fixtures/utils/container";
import { AsyncState } from "@/lib/async-state";

let stub: IRendererWorkerStub;
let TextureRenderService: typeof import("./texture-render.service").TextureRenderService;

beforeAll(async () => {
  // The worker entry reads `import.meta.url`, which the test transform cannot, and the thread is what is stubbed.
  jest.doMock("@xrf/renderer/worker", () => ({ createRendererWorker: () => stub.worker }));
  // jsdom has no offscreen canvas; the client only hands the result to the worker.
  HTMLCanvasElement.prototype.transferControlToOffscreen = function () {
    return {} as OffscreenCanvas;
  };

  ({ TextureRenderService } = await import("./texture-render.service"));
});

beforeEach(() => {
  stub = createRendererWorkerStub();
});

function mockService(): {
  container: Container;
  service: InstanceType<typeof TextureRenderService>;
  viewService: TextureViewService;
} {
  const container: Container = mockContainer([TextureViewService, TextureSurfaceService, TextureRenderService]);

  return {
    container,
    service: container.get(TextureRenderService),
    viewService: container.get(TextureViewService),
  };
}

function mockFile(): ITextureSurfaceFile {
  return { bytes: new ArrayBuffer(16), height: 4, isDecoded: false, width: 4 };
}

function setFiles(container: Container, files: ITextureSurfaceFiles): void {
  runInAction(() => {
    container.get(TextureSurfaceService).files = AsyncState.ready(files);
  });
}

describe("TextureRenderService", () => {
  it("starts nothing until a view attaches or a panel asks", () => {
    mockService();

    expect(stub.requests).toHaveLength(0);
  });

  // A renderer started after a texture was chosen would otherwise show an empty body until something changed.
  it("tells a new renderer what is already open, then shows it on the attached canvas", async () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    await stub.flush();

    expect(stub.requests[0].kind).toBe(ERendererRequest.START);
    expect(stub.take(ERendererRequest.PUT_GEOMETRY)).toHaveLength(1);
    expect(stub.take(ERendererRequest.PUT_SURFACE).map((it) => it.key)).toEqual(["edge", "face"]);
    expect(stub.take(ERendererRequest.PUT_OBJECT)).toHaveLength(1);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(1);
    expect(stub.requests.at(-1)?.kind).toBe(ERendererRequest.ATTACH_VIEW);

    service.dispose();
  });

  it("hands the renderer a copy of each file, so the surface keeps its own bytes", async () => {
    const { service, container } = mockService();
    const base: ITextureSurfaceFile = mockFile();

    service.attach(document.createElement("div"));
    setFiles(container, { ...EMPTY_TEXTURE_SURFACE, base });
    await stub.flush();

    const [put] = stub.take(ERendererRequest.PUT_TEXTURE);

    expect(put.key).toBe("base");
    expect(put.source.encoding).toBe(ERendererTextureEncoding.DDS);
    expect(put.source.bytes).not.toBe(base.bytes);
    expect(put.source.bytes.byteLength).toBe(16);
    expect(stub.take(ERendererRequest.PUT_SURFACE).at(-1)?.surface.textures.base).toBe("base");

    service.dispose();
  });

  it("draws the alpha reading as the engine's draw", async () => {
    const { service, container, viewService } = mockService();

    service.attach(document.createElement("div"));
    setFiles(container, { ...EMPTY_TEXTURE_SURFACE, base: mockFile() });
    viewService.setOptions({ ...viewService.options, alpha: ETextureSurfaceAlpha.BLENDED });
    await stub.flush();

    expect(stub.take(ERendererRequest.PUT_SURFACE).at(-1)?.surface.draw).toBe(ERendererDraw.BLENDED);

    service.dispose();
  });

  it("keeps the renderer and its uploads when the view goes, and lets it go on deactivation", async () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    service.detach();
    await stub.flush();

    expect(stub.requests.at(-1)?.kind).toBe(ERendererRequest.DETACH_VIEW);
    expect(stub.isTerminated()).toBe(false);

    service.dispose();

    expect(stub.take(ERendererRequest.DISPOSE)).toHaveLength(1);
    expect(stub.isTerminated()).toBe(true);
  });

  // What the drag swung to is the view's to keep: the toolbar shows the number the body is lit by.
  it("keeps what a drag over the body swung the light to", async () => {
    const { service, viewService } = mockService();

    service.attach(document.createElement("div"));
    service.dragLight(10, 0);
    await stub.flush();

    expect(viewService.lighting.sunAzimuth).not.toBe(DEFAULT_TEXTURE_LIGHTING.sunAzimuth);
    expect(stub.take(ERendererRequest.LIGHTING)).toHaveLength(2);

    service.dispose();
  });

  it("says what its frames cost, and nothing once no view is drawing", () => {
    const { service } = mockService();

    service.attach(document.createElement("div"));
    stub.respond({
      kind: ERendererResponse.REPORT,
      report: { frame: { ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 } } as IRendererReport,
    });

    expect(service.frameCost.framesPerSecond).toBe(144);
    expect(service.isOffscreen).toBe(true);

    service.detach();

    expect(service.frameCost).toBe(EMPTY_RENDER_FRAME_COST);

    service.dispose();
  });

  it("draws a bump plane without a view, and nothing without a pair", async () => {
    const { service, container } = mockService();

    await expect(service.captureBumpPlane(ERendererBumpPlane.NORMAL, 8, 8)).resolves.toBeNull();
    expect(stub.requests).toHaveLength(0);

    setFiles(container, { ...EMPTY_TEXTURE_SURFACE, bump: { bump: mockFile(), companion: mockFile() } });

    const captured: Promise<unknown> = service.captureBumpPlane(ERendererBumpPlane.NORMAL, 8, 4);

    await stub.flush();

    const [request] = stub.take(ERendererRequest.CAPTURE);

    expect(stub.take(ERendererRequest.ATTACH_VIEW)).toHaveLength(0);
    expect(request.source).toEqual({
      bump: "bump",
      companion: "bump#",
      height: 4,
      kind: ERendererCaptureSource.BUMP_PLANE,
      plane: ERendererBumpPlane.NORMAL,
      width: 8,
    });

    stub.respond({ id: request.id, image: null, kind: ERendererResponse.CAPTURED });

    await expect(captured).resolves.toBeNull();

    service.dispose();
  });

  it("leaves no canvas behind when it is detached", () => {
    const { service } = mockService();
    const element: HTMLElement = document.createElement("div");

    service.attach(element);
    service.attach(element);

    expect(element.querySelectorAll("canvas")).toHaveLength(1);

    service.detach();

    expect(element.querySelectorAll("canvas")).toHaveLength(0);

    service.dispose();
  });

  it("answers the viewport controls before anything is attached", () => {
    const { service } = mockService();

    expect(() => {
      service.dolly(2);
      service.reset();
      service.dragLight(1, 1);
      service.detach();
    }).not.toThrow();
  });
});
