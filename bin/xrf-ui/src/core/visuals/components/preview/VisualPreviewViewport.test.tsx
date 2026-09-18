import { beforeAll, describe, expect, it, jest } from "@jest/globals";
import { render } from "@testing-library/react";

import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS } from "@/core/visuals/components/scene";
import { IVisualModelViews } from "@/core/visuals/lib/visual-views";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";
import { Nullable } from "@/lib/types/general";

const createScene = jest.fn((initialModel: Nullable<IVisualModelViews>) => ({
  initialModel,
  mount: jest.fn(),
  dispose: jest.fn(),
  setModel: jest.fn(),
  applyViewOptions: jest.fn(),
  setDetailLevel: jest.fn(),
  setHighlightedJoint: jest.fn(),
  setPose: jest.fn(),
  setHiddenBones: jest.fn(),
}));

let VisualPreviewViewport: typeof import("./VisualPreviewViewport").VisualPreviewViewport;

beforeAll(async () => {
  // Load the viewport after replacing its WebGL boundary; jsdom cannot create a renderer.
  jest.doMock("@/core/visuals/components/scene", () => ({ VisualPreviewScene: createScene }));
  ({ VisualPreviewViewport } = await import("./VisualPreviewViewport"));
});

function getScene(index: number): ReturnType<typeof createScene> {
  const result = createScene.mock.results[index];

  if (result.type !== "return") {
    throw new Error("Expected scene construction to succeed");
  }

  return result.value;
}

describe("VisualPreviewViewport", () => {
  it("initializes the model and toolbar settings once", () => {
    const model = mockVisualModelViews();
    const options = { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isWireframe: true };
    const { unmount } = render(<VisualPreviewViewport model={model} options={options} detail={0.5} />);
    const scene = getScene(0);

    expect(createScene).toHaveBeenCalledTimes(1);
    expect(createScene).toHaveBeenCalledWith(null);
    expect(scene.mount).toHaveBeenCalledTimes(1);
    expect(scene.setModel.mock.calls).toEqual([[model]]);
    expect(scene.applyViewOptions.mock.calls).toEqual([[options]]);
    expect(scene.setDetailLevel.mock.calls).toEqual([[0.5]]);

    unmount();

    expect(scene.dispose).toHaveBeenCalledTimes(1);
  });

  it("replaces the model in the existing scene without resending unchanged toolbar settings", () => {
    const first = mockVisualModelViews();
    const second = mockVisualModelViews();
    const options = { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isWireframe: true };
    const { rerender } = render(<VisualPreviewViewport model={first} options={options} detail={0.5} />);
    const scene = getScene(0);

    scene.setModel.mockClear();
    scene.applyViewOptions.mockClear();
    scene.setDetailLevel.mockClear();

    rerender(<VisualPreviewViewport model={second} options={options} detail={0.5} />);

    expect(createScene).toHaveBeenCalledTimes(1);
    expect(scene.dispose).not.toHaveBeenCalled();
    expect(scene.setModel.mock.calls).toEqual([[second]]);
    expect(scene.applyViewOptions).not.toHaveBeenCalled();
    expect(scene.setDetailLevel).not.toHaveBeenCalled();
  });

  it("restores the model and toolbar settings after a Strict Mode effect remount", () => {
    const model = mockVisualModelViews();
    const options = { ...DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS, isWireframe: true };
    const { unmount } = render(<VisualPreviewViewport model={model} options={options} detail={0.5} />, {
      reactStrictMode: true,
    });

    expect(createScene).toHaveBeenCalledTimes(2);

    const discarded = getScene(0);
    const active = getScene(1);

    expect(discarded.dispose).toHaveBeenCalledTimes(1);
    expect(active.mount).toHaveBeenCalledTimes(1);
    expect(active.setModel.mock.calls).toEqual([[model]]);
    expect(active.applyViewOptions.mock.calls).toEqual([[options]]);
    expect(active.setDetailLevel.mock.calls).toEqual([[0.5]]);
    expect(active.dispose).not.toHaveBeenCalled();

    unmount();

    expect(active.dispose).toHaveBeenCalledTimes(1);
  });
});
