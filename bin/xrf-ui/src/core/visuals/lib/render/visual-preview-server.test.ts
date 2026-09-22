import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";

import { OffscreenRenderTarget } from "@/core/render/lib/frame/offscreen-render-target";
import { EMPTY_RENDER_FRAME_COST } from "@/core/render/lib/frame/render-frame-cost";
import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import {
  EVisualPreviewRequest,
  EVisualPreviewResponse,
  TVisualPreviewResponse,
} from "@/core/visuals/lib/render/visual-preview-messages";
import { BIND_POSE } from "@/core/visuals/lib/render/visual-render-source";
import { DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS } from "@/core/visuals/lib/scene";
import { mockVisualModelViews } from "@/fixtures/mocks/visual.mocks";

const scene = {
  applyBump: jest.fn(),
  applyTexture: jest.fn(),
  applyViewOptions: jest.fn(),
  dispose: jest.fn(),
  dolly: jest.fn(),
  resetCamera: jest.fn(),
  setDetailLevel: jest.fn(),
  setFrameRateLimit: jest.fn(),
  setHiddenBones: jest.fn(),
  setHighlightedJoint: jest.fn(),
  setLighting: jest.fn(),
  setModel: jest.fn(),
  setPose: jest.fn(),
  setReporter: jest.fn(),
};

const SIZE = { height: 540, pixelRatio: 1, width: 960 };

let VisualPreviewServer: typeof import("./visual-preview-server").VisualPreviewServer;

function mockServer(): { server: InstanceType<typeof VisualPreviewServer>; said: Array<TVisualPreviewResponse> } {
  const said: Array<TVisualPreviewResponse> = [];
  const target: OffscreenRenderTarget = new OffscreenRenderTarget({} as OffscreenCanvas, SIZE);
  const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());

  return { said, server: new VisualPreviewServer(target, element, (response) => said.push(response)) };
}

beforeAll(async () => {
  jest.doMock("@/core/visuals/lib/scene/VisualPreviewScene", () => ({ VisualPreviewScene: jest.fn(() => scene) }));

  ({ VisualPreviewServer } = await import("./visual-preview-server"));
});

describe("VisualPreviewServer", () => {
  beforeEach(() => {
    for (const mock of Object.values(scene)) {
      mock.mockClear();
    }
  });

  it("carries what it is told to the scene it draws with", () => {
    const { server } = mockServer();
    const model = mockVisualModelViews();

    server.take({ kind: EVisualPreviewRequest.MODEL, model });
    server.take({ kind: EVisualPreviewRequest.OPTIONS, options: DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS });
    server.take({ detail: 0.5, kind: EVisualPreviewRequest.DETAIL });
    server.take({ bones: new Set([3]), kind: EVisualPreviewRequest.HIDDEN_BONES });
    server.take({ kind: EVisualPreviewRequest.JOINT, position: [1, 2, 3] });
    server.take({ kind: EVisualPreviewRequest.DOLLY, step: 2 });
    server.take({ kind: EVisualPreviewRequest.RESET });

    expect(scene.setModel).toHaveBeenCalledWith(model);
    expect(scene.applyViewOptions).toHaveBeenCalledWith(DEFAULT_VISUAL_PREVIEW_VIEW_OPTIONS);
    expect(scene.setDetailLevel).toHaveBeenCalledWith(0.5);
    expect(scene.setHiddenBones).toHaveBeenCalledWith(new Set([3]));
    expect(scene.setHighlightedJoint).toHaveBeenCalledWith([1, 2, 3]);
    expect(scene.dolly).toHaveBeenCalledWith(2);
    expect(scene.resetCamera).toHaveBeenCalled();
  });

  // The pose travels as one value and is applied as three arguments, which is the one place they differ.
  it("stands the model the way the page said", () => {
    const { server } = mockServer();
    const transforms: Float32Array = new Float32Array(12);

    server.take({ kind: EVisualPreviewRequest.POSE, pose: { floatsPerBone: 12, frame: 4, transforms } });

    expect(scene.setPose).toHaveBeenCalledWith(transforms, 4, 12);

    server.take({ kind: EVisualPreviewRequest.POSE, pose: BIND_POSE });

    expect(scene.setPose).toHaveBeenCalledWith(null, 0, 0);
  });

  it("says what frames cost, for a readout it cannot draw", () => {
    const { said } = mockServer();

    const report = scene.setReporter.mock.calls[0][0] as (cost: typeof EMPTY_RENDER_FRAME_COST) => void;

    report({ ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 });

    expect(said).toContainEqual({
      cost: { ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 },
      kind: EVisualPreviewResponse.REPORT,
    });
  });

  it("stops reporting when it is released", () => {
    const { server } = mockServer();

    server.dispose();

    expect(scene.setReporter).toHaveBeenLastCalledWith(null);
    expect(scene.dispose).toHaveBeenCalledTimes(1);
  });
});
