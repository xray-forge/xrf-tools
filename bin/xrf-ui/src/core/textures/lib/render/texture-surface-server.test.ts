import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";

import { OffscreenRenderTarget } from "@/core/render/lib/frame/offscreen-render-target";
import { EMPTY_RENDER_FRAME_COST } from "@/core/render/lib/frame/render-frame-cost";
import { RenderProxyElement } from "@/core/render/lib/worker/render-proxy-element";
import {
  ETextureSurfaceRequest,
  ETextureSurfaceResponse,
  TTextureSurfaceResponse,
} from "@/core/textures/lib/render/texture-surface-messages";
import { DEFAULT_TEXTURE_LIGHTING } from "@/core/textures/lib/scene/texture-lighting";
import { EMPTY_TEXTURE_SURFACE, ETextureSurfaceAlpha, ETextureSurfaceShape } from "@/core/textures/lib/texture-surface";

const scene = {
  dispose: jest.fn(),
  dolly: jest.fn(),
  dragLight: jest.fn((_deltaX: number, _deltaY: number) => ({ ...DEFAULT_TEXTURE_LIGHTING, sunAzimuth: 42 })),
  reset: jest.fn(),
  setFrameRateLimit: jest.fn(),
  setLighting: jest.fn(),
  setOptions: jest.fn(),
  setReporter: jest.fn(),
  setTextures: jest.fn(),
};

const SIZE = { height: 540, pixelRatio: 1, width: 960 };

let TextureSurfaceServer: typeof import("./texture-surface-server").TextureSurfaceServer;

beforeAll(async () => {
  jest.doMock("@/core/textures/lib/scene/TextureSurfaceScene", () => ({ TextureSurfaceScene: jest.fn(() => scene) }));

  ({ TextureSurfaceServer } = await import("./texture-surface-server"));
});

function mockServer(): {
  server: InstanceType<typeof TextureSurfaceServer>;
  said: Array<TTextureSurfaceResponse>;
} {
  const said: Array<TTextureSurfaceResponse> = [];
  const target: OffscreenRenderTarget = new OffscreenRenderTarget({} as OffscreenCanvas, SIZE);
  const element: RenderProxyElement = new RenderProxyElement(SIZE, jest.fn());

  return { said, server: new TextureSurfaceServer(target, element, (response) => said.push(response)) };
}

describe("TextureSurfaceServer", () => {
  beforeEach(() => {
    for (const mock of Object.values(scene)) {
      mock.mockClear();
    }
  });

  it("carries what it is told to the scene it draws with", () => {
    const { server } = mockServer();

    server.take({ files: EMPTY_TEXTURE_SURFACE, kind: ETextureSurfaceRequest.TEXTURES });
    server.take({
      kind: ETextureSurfaceRequest.OPTIONS,
      options: {
        alpha: ETextureSurfaceAlpha.CUT_OUT,
        isBumped: true,
        isLit: true,
        shape: ETextureSurfaceShape.SPHERE,
        tiling: 2,
      },
    });
    server.take({ kind: ETextureSurfaceRequest.LIGHTING, lighting: DEFAULT_TEXTURE_LIGHTING });
    server.take({ kind: ETextureSurfaceRequest.FRAME_RATE, limit: "30" });
    server.take({ kind: ETextureSurfaceRequest.DOLLY, step: 2 });
    server.take({ kind: ETextureSurfaceRequest.RESET });

    expect(scene.setTextures).toHaveBeenCalledWith(EMPTY_TEXTURE_SURFACE);
    expect(scene.setOptions).toHaveBeenCalledWith(expect.objectContaining({ tiling: 2 }));
    expect(scene.setLighting).toHaveBeenCalledWith(DEFAULT_TEXTURE_LIGHTING);
    expect(scene.setFrameRateLimit).toHaveBeenCalledWith("30");
    expect(scene.dolly).toHaveBeenCalledWith(2);
    expect(scene.reset).toHaveBeenCalled();
  });

  // The drag is gathered where the pointer is and applied where the light is, so the answer has to come back.
  it("answers a drag with where it put the light", () => {
    const { server, said } = mockServer();

    server.take({ deltaX: 10, deltaY: 4, kind: ETextureSurfaceRequest.DRAG_LIGHT });

    expect(scene.dragLight).toHaveBeenCalledWith(10, 4);
    expect(said).toContainEqual({
      kind: ETextureSurfaceResponse.LIGHTING,
      lighting: { ...DEFAULT_TEXTURE_LIGHTING, sunAzimuth: 42 },
    });
  });

  it("says what frames cost, for a readout it cannot draw", () => {
    const { said } = mockServer();

    const report = scene.setReporter.mock.calls[0][0] as (cost: typeof EMPTY_RENDER_FRAME_COST) => void;

    report({ ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 });

    expect(said).toContainEqual({
      cost: { ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 },
      kind: ETextureSurfaceResponse.REPORT,
    });
  });

  it("stops reporting when it is released", () => {
    const { server } = mockServer();

    server.dispose();

    expect(scene.setReporter).toHaveBeenLastCalledWith(null);
    expect(scene.dispose).toHaveBeenCalledTimes(1);
  });
});
