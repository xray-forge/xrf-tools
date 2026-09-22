import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";

import { EMPTY_RENDER_FRAME_COST } from "@/core/render/lib/frame/render-frame-cost";
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

  return { said, server: new TextureSurfaceServer((response: TTextureSurfaceResponse) => said.push(response)) };
}

function started(server: InstanceType<typeof TextureSurfaceServer>): void {
  server.take({
    canvas: {} as OffscreenCanvas,
    height: 540,
    kind: ETextureSurfaceRequest.START,
    pixelRatio: 1,
    width: 960,
  });
}

describe("TextureSurfaceServer", () => {
  beforeEach(() => {
    for (const mock of Object.values(scene)) {
      mock.mockClear();
    }
  });

  // Every message may arrive before the canvas does, and a server that threw on one would take the worker down.
  it("takes anything said before it has somewhere to draw", () => {
    const { server, said } = mockServer();

    expect(() => {
      server.take({ files: EMPTY_TEXTURE_SURFACE, kind: ETextureSurfaceRequest.TEXTURES });
      server.take({ height: 1, kind: ETextureSurfaceRequest.RESIZE, pixelRatio: 1, width: 1 });
      server.take({ kind: ETextureSurfaceRequest.DOLLY, step: 2 });
      server.take({ kind: ETextureSurfaceRequest.RESET });
      server.take({ deltaX: 1, deltaY: 1, kind: ETextureSurfaceRequest.DRAG_LIGHT });
      server.take({ kind: ETextureSurfaceRequest.DISPOSE });
    }).not.toThrow();

    expect(said).toEqual([]);
  });

  it("carries what it is told to the scene it started", () => {
    const { server } = mockServer();

    started(server);

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

    started(server);
    server.take({ deltaX: 10, deltaY: 4, kind: ETextureSurfaceRequest.DRAG_LIGHT });

    expect(scene.dragLight).toHaveBeenCalledWith(10, 4);
    expect(said).toContainEqual({
      kind: ETextureSurfaceResponse.LIGHTING,
      lighting: { ...DEFAULT_TEXTURE_LIGHTING, sunAzimuth: 42 },
    });
  });

  it("says what frames cost, for a readout it cannot draw", () => {
    const { server, said } = mockServer();

    started(server);

    const report = scene.setReporter.mock.calls[0][0] as (cost: typeof EMPTY_RENDER_FRAME_COST) => void;

    report({ ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 });

    expect(said).toContainEqual({
      cost: { ...EMPTY_RENDER_FRAME_COST, framesPerSecond: 144 },
      kind: ETextureSurfaceResponse.REPORT,
    });
  });

  it("releases the scene it started", () => {
    const { server } = mockServer();

    started(server);
    server.take({ kind: ETextureSurfaceRequest.DISPOSE });

    expect(scene.dispose).toHaveBeenCalledTimes(1);

    // Nothing is drawing any more, and saying so again is not an error.
    server.take({ kind: ETextureSurfaceRequest.RESET });

    expect(scene.reset).not.toHaveBeenCalled();
  });
});
