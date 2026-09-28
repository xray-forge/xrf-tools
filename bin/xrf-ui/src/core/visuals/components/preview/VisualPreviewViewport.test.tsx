import { beforeAll, beforeEach, describe, expect, it } from "@jest/globals";
import { act, RenderResult } from "@testing-library/react";
import { Binding } from "@wirestate/core";
import { makeAutoObservable, runInAction } from "@wirestate/mobx";
import { ERendererRequest, ERendererResponse } from "@xrf/renderer";
import { createRendererWorkerStub, IRendererWorkerStub } from "@xrf/renderer/fixtures";

import { IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { mockVisualModelViews, mockVisualSubmeshViews } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";
import { mockRendererThread } from "@/fixtures/utils/renderer";

let stubs: Array<IRendererWorkerStub>;
let VisualPreviewViewport: typeof import("./VisualPreviewViewport").VisualPreviewViewport;
let bindings: Array<Binding>;
let source: IVisualRenderSource;

beforeAll(async () => {
  mockRendererThread((): Worker => {
    const stub: IRendererWorkerStub = createRendererWorkerStub();

    stubs.push(stub);

    return stub.worker;
  });

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

  it("covers the viewport and says why when the renderer fails", async () => {
    runInAction(() => (source.model = mockVisualModelViews({ submeshes: [mockVisualSubmeshViews()] })));

    const view: RenderResult = renderWithProviders(<VisualPreviewViewport />, { bindings });

    await stubs[0].flush();
    act(() => stubs[0].respond({ kind: ERendererResponse.FAILED, reason: "No WebGPU adapter" }));

    expect(view.getByTestId("render-failure-cover")).toHaveTextContent("No WebGPU adapter");
    expect(view.queryByTestId("render-frame-readout")).not.toBeInTheDocument();

    view.unmount();
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
