import { beforeAll, describe, expect, it, jest } from "@jest/globals";
import { act } from "@testing-library/react";
import { Binding } from "@wirestate/core";
import { makeAutoObservable, runInAction } from "@wirestate/mobx";

import { DomRenderTarget } from "@/core/render/lib/frame/dom-render-target";
import { IVisualRenderSource, VISUAL_RENDER_SOURCE } from "@/core/visuals/lib/render";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { renderWithProviders } from "@/fixtures/utils/render";

const createScene = jest.fn((_target: unknown, _element: unknown, _model: unknown) => ({
  applyBump: jest.fn(),
  applyTexture: jest.fn(),
  applyViewOptions: jest.fn(),
  dispose: jest.fn(),
  setReporter: jest.fn(),
  dolly: jest.fn(),
  resetCamera: jest.fn(),
  setDetailLevel: jest.fn(),
  setFrameRateLimit: jest.fn(),
  setHiddenBones: jest.fn(),
  setHighlightedJoint: jest.fn(),
  setLighting: jest.fn(),
  setModel: jest.fn(),
  setPose: jest.fn(),
}));

let VisualPreviewViewport: typeof import("./VisualPreviewViewport").VisualPreviewViewport;
let bindings: Array<Binding>;
let source: IVisualRenderSource;

beforeAll(async () => {
  // Loaded after its WebGL boundary is replaced; jsdom cannot create a renderer. The service is what builds the
  // scene now, so it has to come through the same mock.
  jest.doMock("@/core/visuals/lib/scene", () => ({ VisualPreviewScene: createScene }));

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

function getScene(index: number): ReturnType<typeof createScene> {
  const result = createScene.mock.results[index];

  if (result.type !== "return") {
    throw new Error("Expected scene construction to succeed");
  }

  return result.value;
}

describe("VisualPreviewViewport", () => {
  it("builds one scene and tells it what is already open", () => {
    const model = mockVisualModelViews();

    runInAction(() => (source.model = model));

    const { unmount } = renderWithProviders(<VisualPreviewViewport />, { bindings });
    const scene = getScene(0);

    expect(createScene).toHaveBeenCalledTimes(1);
    // Where to draw, what to listen to, and nothing open yet: on a page the first two are one element.
    expect(createScene).toHaveBeenCalledWith(expect.any(DomRenderTarget), expect.any(HTMLCanvasElement), null);
    expect(scene.setModel.mock.calls).toEqual([[model]]);
    expect(scene.applyViewOptions).toHaveBeenCalledTimes(1);
    expect(scene.setDetailLevel.mock.calls).toEqual([[0]]);

    unmount();

    expect(scene.dispose).toHaveBeenCalledTimes(1);
  });

  // Clicking through a tree keeps the camera and the webgl context: only the geometry is replaced.
  it("replaces the model in the scene it already has", () => {
    const second = mockVisualModelViews();

    runInAction(() => (source.model = mockVisualModelViews()));

    renderWithProviders(<VisualPreviewViewport />, { bindings });

    const scene = getScene(0);

    scene.setModel.mockClear();
    scene.applyViewOptions.mockClear();

    act(() => runInAction(() => (source.model = second)));

    expect(createScene).toHaveBeenCalledTimes(1);
    expect(scene.dispose).not.toHaveBeenCalled();
    expect(scene.setModel.mock.calls).toEqual([[second]]);
    expect(scene.applyViewOptions).not.toHaveBeenCalled();
  });

  it("rebuilds a whole scene after a strict mode remount rather than leaking the first", () => {
    const model = mockVisualModelViews();

    runInAction(() => (source.model = model));

    const { unmount } = renderWithProviders(<VisualPreviewViewport />, { bindings, isStrict: true });

    expect(createScene).toHaveBeenCalledTimes(2);

    const discarded = getScene(0);
    const active = getScene(1);

    expect(discarded.dispose).toHaveBeenCalledTimes(1);
    expect(active.setModel.mock.calls).toEqual([[model]]);
    expect(active.dispose).not.toHaveBeenCalled();

    unmount();

    expect(active.dispose).toHaveBeenCalledTimes(1);
  });
});
