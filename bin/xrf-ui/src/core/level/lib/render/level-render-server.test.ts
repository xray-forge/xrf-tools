import { describe, expect, it } from "@jest/globals";

import { EMPTY_LEVEL_FLY_INPUT } from "@/core/level/lib/camera/level-fly-input";
import { ILevelFlyMotion } from "@/core/level/lib/camera/level-fly-motion";
import {
  ELevelRenderRequest,
  ELevelRenderResponse,
  TLevelRenderResponse,
} from "@/core/level/lib/render/level-render-messages";
import { LevelRenderServer } from "@/core/level/lib/render/level-render-server";

function mockServer(): { server: LevelRenderServer; said: Array<TLevelRenderResponse> } {
  const said: Array<TLevelRenderResponse> = [];

  return { said, server: new LevelRenderServer((response: TLevelRenderResponse) => said.push(response)) };
}

function motion(lookX: number, isForward: boolean = false): ILevelFlyMotion {
  return {
    keys: { ...EMPTY_LEVEL_FLY_INPUT, forward: isForward },
    lookX,
    lookY: 0,
  };
}

describe("LevelRenderServer", () => {
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

  // Every message may arrive before the canvas does, and a renderer that threw on one would take the worker
  // down with it.
  it("takes anything said before it has somewhere to draw", () => {
    const { server, said } = mockServer();

    expect(() => {
      server.take({ kind: ELevelRenderRequest.OPEN, level: null });
      server.take({ change: { delivered: [], released: null }, kind: ELevelRenderRequest.DELIVER });
      server.take({ change: { delivered: [], retained: null }, kind: ELevelRenderRequest.SUPPLY });
      server.take({ height: 1, kind: ELevelRenderRequest.RESIZE, pixelRatio: 1, width: 1 });
      server.take({ kind: ELevelRenderRequest.DISPOSE });
    }).not.toThrow();

    expect(said).toEqual([]);
  });

  // Whoever asked is waiting on the number they asked with, so an answer has to come back even when there is
  // nothing to measure.
  it("answers a measurement it cannot make", () => {
    const { server, said } = mockServer();

    server.take({ id: 7, kind: ELevelRenderRequest.MEASURE });

    expect(said).toEqual([{ geometry: new Map(), id: 7, kind: ELevelRenderResponse.MEASURED }]);
  });
});
