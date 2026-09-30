import { describe, expect, it, jest } from "@jest/globals";
import { IRendererObject } from "@xrf/renderer";

import { LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import { LevelRenderSpawnSet, TLevelRenderSpawnSink } from "@/core/level/lib/render/level-render-spawn-set";
import { toLevelSurfaceColor } from "@/core/level/lib/render/level-render-surface";
import { mockLevelSpawnModel, mockLevelSpawnObject } from "@/fixtures/mocks/level.mocks";
import { mockVisualTransform } from "@/fixtures/mocks/visual.mocks";

type TMockSink = { [K in keyof TLevelRenderSpawnSink]: jest.Mock<TLevelRenderSpawnSink[K]> };

function mockSink(): TMockSink {
  return {
    putGeometry: jest.fn(),
    putObject: jest.fn(),
    putSurface: jest.fn(),
    releaseGeometry: jest.fn(),
    releaseObject: jest.fn(),
    releaseSurface: jest.fn(),
  };
}

function mockObjects(): LevelSpawnObjectsDescription {
  return {
    objects: [
      mockLevelSpawnObject({ index: 0, transform: mockVisualTransform({ x: 1, y: 0, z: 0 }) }),
      mockLevelSpawnObject({ index: 1, name: "lamp", visual: 1 }),
      mockLevelSpawnObject({ index: 2, transform: mockVisualTransform({ x: 2, y: 0, z: 0 }) }),
    ],
    visuals: ["crate", "lamp"],
  };
}

function listPut(sink: TMockSink): Array<string> {
  return sink.putObject.mock.calls.map(([key]) => key);
}

describe("LevelRenderSpawnSet", () => {
  it("stands a visual once in every place an object of it stands, dressed with its base and a colour of its own", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);
    const key: string = LEVEL_RENDER_KEYS.spawn(0, 0);

    set.stand({ models: new Map([[0, mockLevelSpawnModel("crate")]]), objects: mockObjects() });

    const [, object] = sink.putObject.mock.calls[0] as [string, IRendererObject];

    expect(listPut(sink)).toEqual([key]);
    // Two crates, sixteen floats each, the second two metres along x.
    expect(object.instances?.transforms).toHaveLength(32);
    expect(object.instances?.transforms?.[28]).toBe(2);
    expect(sink.putSurface.mock.calls[0]).toEqual([
      key,
      expect.objectContaining({ color: toLevelSurfaceColor(0), textures: { base: "crate" } }),
    ]);
  });

  // The loader hands on every model read so far with each batch, so a batch puts only what it brought.
  it("puts only the models a later delivery of the same objects brings", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);
    const objects: LevelSpawnObjectsDescription = mockObjects();
    const crate: ILevelSpawnModel = mockLevelSpawnModel("crate");

    set.stand({ models: new Map([[0, crate]]), objects });
    set.stand({
      models: new Map([
        [0, crate],
        [1, mockLevelSpawnModel("lamp")],
      ]),
      objects,
    });

    expect(listPut(sink)).toEqual([LEVEL_RENDER_KEYS.spawn(0, 0), LEVEL_RENDER_KEYS.spawn(1, 0)]);
    expect(sink.releaseObject).not.toHaveBeenCalled();
  });

  it("lets every part go for other objects, and for none", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);
    const crate: ILevelSpawnModel = mockLevelSpawnModel("crate");

    set.stand({ models: new Map([[0, crate]]), objects: mockObjects() });
    set.stand({ models: new Map([[0, crate]]), objects: mockObjects() });

    expect(sink.releaseObject.mock.calls).toEqual([[LEVEL_RENDER_KEYS.spawn(0, 0)]]);
    expect(listPut(sink)).toHaveLength(2);

    set.stand(null);

    expect(sink.releaseObject).toHaveBeenCalledTimes(2);
    expect(sink.releaseGeometry).toHaveBeenCalledTimes(2);
    expect(sink.releaseSurface).toHaveBeenCalledTimes(2);
  });
});
