import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { Container } from "@wirestate/core";
import { makeAutoObservable, runInAction } from "@wirestate/mobx";
import { ERendererOverlay, ERendererRequest, TRendererRequest } from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";

import { BIND_POSE, IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { IVisualTextureFile } from "@/core/visuals/lib/visual-texture";
import { VisualViewService } from "@/core/visuals/services/visual-view.service";
import { mockVisualModelViews, mockVisualSubmeshViews } from "@/fixtures/mocks/visual.mocks";
import { mockContainer } from "@/fixtures/utils/container";

let stub: IRendererWorkerStub;
let VisualRenderService: typeof import("./visual-render.service").VisualRenderService;

beforeAll(async () => {
  // The worker entry reads `import.meta.url`, which the test transform cannot, and the thread is what is stubbed.
  jest.doMock("@xrf/renderer/worker", () => ({ createRendererWorker: () => stub.worker }));
  HTMLCanvasElement.prototype.transferControlToOffscreen = function () {
    return {} as OffscreenCanvas;
  };

  ({ VisualRenderService } = await import("./visual-render.service"));
});

beforeEach(() => {
  stub = createRendererWorkerStub();
});

function mockTextureFile(): IVisualTextureFile {
  return { bytes: new ArrayBuffer(8), isAlphaRead: false, isDecoded: false, logicalPath: "textures\\wall" };
}

function mockSource(overrides: Partial<IVisualRenderSource> = {}): IVisualRenderSource {
  return makeAutoObservable<IVisualRenderSource>(
    { bumps: new Map(), model: null, textures: new Map(), ...overrides },
    {},
    { deep: false }
  );
}

function mockSkinnedModel() {
  return mockVisualModelViews({
    skeletonBinds: new Float32Array(24),
    skeletonPairs: new Uint16Array([1, 0]),
    submeshes: [mockVisualSubmeshViews({ skinIndices: new Uint16Array(12), skinWeights: new Float32Array(12) })],
  });
}

function mockAttached(source: IVisualRenderSource): {
  service: InstanceType<typeof VisualRenderService>;
  viewService: VisualViewService;
} {
  const container: Container = mockContainer([
    VisualViewService,
    VisualRenderService,
    { factory: () => source, token: VISUAL_RENDER_SOURCE },
  ]);
  const service = container.get(VisualRenderService);

  service.attach(document.createElement("div"));

  return { service, viewService: container.get(VisualViewService) };
}

describe("VisualRenderService", () => {
  it("dresses a model published together with its textures, uploading each file once", () => {
    const file: IVisualTextureFile = mockTextureFile();
    const model = mockVisualModelViews({
      submeshes: [mockVisualSubmeshViews({ index: 0 }), mockVisualSubmeshViews({ index: 1 })],
    });
    const { service } = mockAttached(
      mockSource({
        model,
        textures: new Map([
          [0, file],
          [1, file],
        ]),
      })
    );

    expect(stub.take(ERendererRequest.PUT_GEOMETRY).map((it) => it.key)).toEqual(["submesh:0", "submesh:1"]);
    expect(stub.take(ERendererRequest.PUT_TEXTURE).filter((it) => it.key === file.logicalPath)).toHaveLength(1);
    expect(stub.take(ERendererRequest.PUT_SURFACE).at(-1)?.surface.textures.base).toBe(file.logicalPath);
    expect(stub.take(ERendererRequest.CAMERA)).toHaveLength(1);
    expect(stub.requests.at(-1)?.kind).toBe(ERendererRequest.ATTACH_VIEW);

    service.dispose();
  });

  it("sends a baked motion once, and only the frame after that", () => {
    const transforms: Float32Array = new Float32Array(48);
    const source: IVisualRenderSource = mockSource({ model: mockSkinnedModel(), pose: BIND_POSE });
    const { service } = mockAttached(source);

    expect(stub.take(ERendererRequest.POSE).at(-1)?.pose).toEqual({ frame: 0, hiddenBones: [], motion: null });

    runInAction(() => (source.pose = { floatsPerBone: 12, frame: 0, transforms }));
    runInAction(() => (source.pose = { floatsPerBone: 12, frame: 1, transforms }));

    expect(stub.take(ERendererRequest.PUT_MOTION)).toHaveLength(1);
    expect(stub.take(ERendererRequest.POSE).at(-1)?.pose).toEqual({ frame: 1, hiddenBones: [], motion: "motion" });

    service.dispose();
  });

  it("collapses the bones the source hides", () => {
    const source: IVisualRenderSource = mockSource({ hiddenBoneIndices: new Set([1]), model: mockSkinnedModel() });
    const { service } = mockAttached(source);

    expect(stub.take(ERendererRequest.POSE).at(-1)?.pose.hiddenBones).toEqual([1]);
    expect(stub.take(ERendererRequest.PUT_OBJECT)[0].object.skeleton).toBe("skeleton");

    service.dispose();
  });

  it("lets the last model's submeshes go when another replaces it", () => {
    const source: IVisualRenderSource = mockSource({
      model: mockVisualModelViews({ submeshes: [mockVisualSubmeshViews()] }),
    });
    const { service } = mockAttached(source);

    runInAction(() => (source.model = mockVisualModelViews()));

    const released: Array<TRendererRequest> = stub.requests.filter(
      (request) =>
        request.kind === ERendererRequest.RELEASE_GEOMETRY || request.kind === ERendererRequest.RELEASE_OBJECT
    );

    expect(released).toHaveLength(2);

    service.dispose();
  });

  it("draws the skeleton overlay and the joint marker only while the skeleton is shown", () => {
    const source: IVisualRenderSource = mockSource({ highlightedJoint: [0, 1, 0], model: mockSkinnedModel() });
    const { service, viewService } = mockAttached(source);

    function overlays(): Array<string> {
      return stub.take(ERendererRequest.PUT_OVERLAY).map((it) => it.overlay.kind);
    }

    expect(overlays()).toEqual([ERendererOverlay.LINES]);

    viewService.setOptions({ ...viewService.options, isSkeletonVisible: true });

    expect(overlays()).toContain(ERendererOverlay.SKELETON);
    expect(overlays()).toContain(ERendererOverlay.POINTS);

    service.dispose();
  });

  it("keeps the renderer when the view goes, and leaves no canvas behind", () => {
    const { service } = mockAttached(mockSource());
    const element: HTMLElement = document.createElement("div");

    service.attach(element);

    expect(element.querySelectorAll("canvas")).toHaveLength(1);

    service.detach();

    expect(element.querySelectorAll("canvas")).toHaveLength(0);
    expect(stub.isTerminated()).toBe(false);

    service.dispose();

    expect(stub.isTerminated()).toBe(true);
  });
});
