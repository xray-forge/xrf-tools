import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { OffscreenRenderTarget } from "@xrf/renderer";

import { EMPTY_LEVEL_FLY_INPUT } from "@/core/level/lib/camera/level-fly-input";
import { ILevelFlyMotion } from "@/core/level/lib/camera/level-fly-motion";
import {
  ELevelRenderRequest,
  ELevelRenderResponse,
  TLevelRenderResponse,
} from "@/core/level/lib/render/level-render-messages";

const scene = {
  deliver: jest.fn(),
  dispose: jest.fn(),
  measureSurfaceGeometry: jest.fn(() => new Map()),
  open: jest.fn(),
  setMotion: jest.fn(),
  setView: jest.fn(),
  supply: jest.fn(),
};

let LevelRenderServer: typeof import("./level-render-server").LevelRenderServer;

function mockServer(): { server: InstanceType<typeof LevelRenderServer>; said: Array<TLevelRenderResponse> } {
  const said: Array<TLevelRenderResponse> = [];
  const target: OffscreenRenderTarget = new OffscreenRenderTarget({} as OffscreenCanvas, {
    height: 540,
    pixelRatio: 1,
    width: 960,
  });

  return { said, server: new LevelRenderServer(target, (response) => said.push(response)) };
}

function motion(lookX: number, isForward: boolean = false): ILevelFlyMotion {
  return { keys: { ...EMPTY_LEVEL_FLY_INPUT, forward: isForward }, lookX, lookY: 0 };
}

beforeAll(async () => {
  jest.doMock("@/core/level/lib/scene", () => ({ LevelPreviewScene: jest.fn(() => scene) }));

  ({ LevelRenderServer } = await import("./level-render-server"));
});

describe("LevelRenderServer", () => {
  beforeEach(() => {
    for (const mock of Object.values(scene)) {
      mock.mockClear();
    }
  });

  it("carries what it is told to the scene it draws with", () => {
    const { server } = mockServer();

    server.take({ kind: ELevelRenderRequest.OPEN, level: null });
    server.take({ change: { delivered: [], released: null }, kind: ELevelRenderRequest.DELIVER });
    server.take({ change: { delivered: [], retained: null }, kind: ELevelRenderRequest.SUPPLY });

    expect(scene.open).toHaveBeenCalledWith(null);
    expect(scene.deliver).toHaveBeenCalled();
    expect(scene.supply).toHaveBeenCalled();
  });

  it("forgets a look once a frame has taken it", () => {
    const { server } = mockServer();

    server.take({ kind: ELevelRenderRequest.MOTION, motion: motion(40) });

    expect(server.drain().lookX).toBe(40);
    expect(server.drain().lookX).toBe(0);
  });

  // Keys are held rather than gathered, so a page that says nothing more is still holding what it last said.
  it("keeps reporting a key until the page says otherwise", () => {
    const { server } = mockServer();

    server.take({ kind: ELevelRenderRequest.MOTION, motion: motion(0, true) });

    expect(server.drain().keys.forward).toBe(true);
    expect(server.drain().keys.forward).toBe(true);

    server.take({ kind: ELevelRenderRequest.MOTION, motion: motion(0, false) });

    expect(server.drain().keys.forward).toBe(false);
  });

  it("reports nothing before the page has said anything", () => {
    const { server } = mockServer();

    expect(server.drain()).toEqual({ keys: expect.any(Object), lookX: 0, lookY: 0 });
  });

  // Whoever asked is waiting on the number they asked with, so an answer has to come back either way.
  it("answers a measurement with the number it was asked with", () => {
    const { server, said } = mockServer();

    server.take({ id: 7, kind: ELevelRenderRequest.MEASURE });

    expect(said).toEqual([{ geometry: new Map(), id: 7, kind: ELevelRenderResponse.MEASURED }]);
  });

  it("is the motion source of the scene it built", () => {
    const { server } = mockServer();

    expect(scene.setMotion).toHaveBeenCalledWith(server);
  });
});
