import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { Binding } from "@wirestate/core";
import { makeAutoObservable, runInAction } from "@wirestate/mobx";
import { ERendererRequest } from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";

import { IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { mockVisualModelViews, mockVisualSubmeshViews } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

let stubs: Array<IRendererWorkerStub>;
let VisualPreviewViewport: typeof import("./VisualPreviewViewport").VisualPreviewViewport;
let bindings: Array<Binding>;
let source: IVisualRenderSource;

beforeAll(async () => {
  // The renderer's thread is what is stubbed; jsdom has neither a GPU nor an offscreen canvas.
  jest.doMock("@xrf/renderer/worker", () => ({
    createRendererWorker: () => {
      const stub: IRendererWorkerStub = createRendererWorkerStub();

      stubs.push(stub);

      return stub.worker;
    },
  }));
  HTMLCanvasElement.prototype.transferControlToOffscreen = function () {
    return {} as OffscreenCanvas;
  };

  const { VisualRenderService } = await import("@/core/visuals/services/visual-render.service");
  const { VisualViewService } = await import("@/core/visuals/services/visual-view.service");

  ({ VisualPreviewViewport } = await import("./VisualPreviewViewport"));

  source = makeAutoObservable<IVisualRenderSource>(
    { bumps: new Map(), model: null, textures: new Map() },
    {},
    { deep: false }
  );

  bindings = [VisualViewService, VisualRenderService, { factory: () => source, token: VISUAL_RENDER_SOURCE }];
});

beforeEach(() => {
  stubs = [];
});

describe("VisualPreviewViewport", () => {
  it("starts one renderer, tells it what is already open, and shows it on its canvas", async () => {
    runInAction(() => (source.model = mockVisualModelViews({ submeshes: [mockVisualSubmeshViews()] })));

    const { unmount } = renderWithProviders(<VisualPreviewViewport />, { bindings });

    await stubs[0].flush();

    expect(stubs).toHaveLength(1);
    expect(stubs[0].take(ERendererRequest.PUT_GEOMETRY)).toHaveLength(1);
    expect(stubs[0].take(ERendererRequest.ATTACH_VIEW)).toHaveLength(1);

    unmount();
    await stubs[0].flush();

    expect(stubs[0].take(ERendererRequest.DETACH_VIEW)).toHaveLength(1);
  });

  // Clicking through a tree keeps the renderer and its camera controller: only the model is replaced.
  it("replaces the model in the renderer it already has", async () => {
    runInAction(() => (source.model = mockVisualModelViews({ submeshes: [mockVisualSubmeshViews()] })));

    renderWithProviders(<VisualPreviewViewport />, { bindings });

    act(() => runInAction(() => (source.model = mockVisualModelViews({ submeshes: [mockVisualSubmeshViews()] }))));
    await stubs[0].flush();

    expect(stubs).toHaveLength(1);
    expect(stubs[0].take(ERendererRequest.RELEASE_GEOMETRY)).toHaveLength(1);
    expect(stubs[0].take(ERendererRequest.PUT_GEOMETRY)).toHaveLength(2);
  });

  it("attaches a fresh canvas after a strict mode remount rather than leaking the first", async () => {
    runInAction(() => (source.model = mockVisualModelViews()));

    const { container, unmount } = renderWithProviders(<VisualPreviewViewport />, { bindings, isStrict: true });

    await stubs[0].flush();

    expect(stubs).toHaveLength(1);
    expect(stubs[0].take(ERendererRequest.ATTACH_VIEW)).toHaveLength(2);
    expect(container.querySelectorAll("canvas")).toHaveLength(1);

    unmount();
  });
});
