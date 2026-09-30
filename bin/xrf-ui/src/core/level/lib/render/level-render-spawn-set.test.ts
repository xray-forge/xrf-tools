import { describe, expect, it, jest } from "@jest/globals";
import { IRendererGeometry, IRendererObject } from "@xrf/renderer";

import { ELevelSpawnCategory, LevelSpawnObjectsDescription } from "@/core/ipc/types/xrf-app";
import { LEVEL_RENDER_KEYS } from "@/core/level/lib/render/level-render-keys";
import { ILevelSpawnDelivery, ILevelSpawnModel } from "@/core/level/lib/render/level-render-protocol";
import { LevelRenderSpawnSet, TLevelRenderSpawnSink } from "@/core/level/lib/render/level-render-spawn-set";
import { toLevelSurfaceColor } from "@/core/level/lib/render/level-render-surface";
import { mockLevelSpawnModel, mockLevelSpawnObject } from "@/fixtures/mocks/level.mocks";
import { mockVisualTransform } from "@/fixtures/mocks/visual.mocks";

type TMockSink = { [K in keyof TLevelRenderSpawnSink]: jest.Mock<TLevelRenderSpawnSink[K]> };

const PROPS: string = LEVEL_RENDER_KEYS.spawnObject(0, ELevelSpawnCategory.PROPS);
const ITEMS: string = LEVEL_RENDER_KEYS.spawnObject(0, ELevelSpawnCategory.ITEMS);
const LAMPS: string = LEVEL_RENDER_KEYS.spawnObject(1, ELevelSpawnCategory.LAMPS);

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

/** Two crates and a bottle standing as the first visual, a lamp as the second. */
function mockObjects(): LevelSpawnObjectsDescription {
  return {
    objects: [
      mockLevelSpawnObject({ index: 0, transform: mockVisualTransform({ x: 1, y: 0, z: 0 }) }),
      mockLevelSpawnObject({ category: ELevelSpawnCategory.LAMPS, index: 1, name: "lamp", visual: 1 }),
      mockLevelSpawnObject({ index: 2, transform: mockVisualTransform({ x: 2, y: 0, z: 0 }) }),
      mockLevelSpawnObject({ category: ELevelSpawnCategory.ITEMS, index: 3, name: "bottle" }),
    ],
    visuals: ["crate", "lamp"],
  };
}

function deliver(
  objects: LevelSpawnObjectsDescription,
  models: Array<[number, ILevelSpawnModel]>,
  hemi: ReadonlyMap<number, ReadonlyArray<number>> = new Map()
): ILevelSpawnDelivery {
  return { hemi, models: new Map(models), objects };
}

function listPut(sink: TMockSink): Array<string> {
  return sink.putObject.mock.calls.map(([key]) => key);
}

describe("LevelRenderSpawnSet", () => {
  it("stands a visual once as one geometry, and each category's objects of it in every place one stands", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);

    set.stand(deliver(mockObjects(), [[0, mockLevelSpawnModel("crate")]]));

    const [, props] = sink.putObject.mock.calls[0] as [string, IRendererObject];

    expect(sink.putGeometry.mock.calls.map(([key]) => key)).toEqual([LEVEL_RENDER_KEYS.spawnGeometry(0)]);
    expect(listPut(sink)).toEqual([PROPS, ITEMS]);
    // Two crates, sixteen floats each, the second two metres along x.
    expect(props.instances?.transforms).toHaveLength(32);
    expect(props.instances?.transforms?.[28]).toBe(2);
    expect(props.surfaces).toEqual([LEVEL_RENDER_KEYS.spawnSurface(0, 0)]);
    expect(sink.putSurface.mock.calls[0]).toEqual([
      LEVEL_RENDER_KEYS.spawnSurface(0, 0),
      expect.objectContaining({ color: toLevelSurfaceColor(0), textures: { base: "crate" } }),
    ]);
  });

  it("joins a model's submeshes into one geometry, a group a submesh, indices moved past the vertices before", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);

    set.stand(deliver(mockObjects(), [[0, mockLevelSpawnModel("crate", 2)]]));

    const [, geometry] = sink.putGeometry.mock.calls[0] as [string, IRendererGeometry];
    const vertices: number = geometry.position.length / 3 / 2;

    expect(geometry.groups).toEqual([
      { count: 3, slot: 0, start: 0 },
      { count: 3, slot: 1, start: 3 },
    ]);
    expect(Array.from(geometry.index ?? []).slice(3)).toEqual([0, 1, 2].map((it) => it + vertices));
    expect(sink.putSurface).toHaveBeenCalledTimes(2);
  });

  it("lights each place by its object's hemisphere cube, and none where any object has none", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);
    const hemi: Map<number, ReadonlyArray<number>> = new Map([
      [0, [1, 2, 3, 4, 5, 6]],
      [2, [7, 8, 9, 10, 11, 12]],
    ]);

    set.stand(deliver(mockObjects(), [[0, mockLevelSpawnModel("crate")]], hemi));

    const [, props] = sink.putObject.mock.calls[0] as [string, IRendererObject];
    const [, items] = sink.putObject.mock.calls[1] as [string, IRendererObject];

    expect(Array.from(props.instances?.hemiCube ?? [])).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    expect(items.instances?.hemiCube).toBeUndefined();
  });

  // The loader hands on every model read so far with each batch, so a batch puts only what it brought.
  it("puts only the models a later delivery of the same objects brings", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);
    const objects: LevelSpawnObjectsDescription = mockObjects();
    const crate: ILevelSpawnModel = mockLevelSpawnModel("crate");

    set.stand(deliver(objects, [[0, crate]]));
    set.stand(
      deliver(objects, [
        [0, crate],
        [1, mockLevelSpawnModel("lamp")],
      ])
    );

    expect(listPut(sink)).toEqual([PROPS, ITEMS, LAMPS]);
    expect(sink.putGeometry).toHaveBeenCalledTimes(2);
    expect(sink.releaseObject).not.toHaveBeenCalled();
  });

  it("lets a hidden category's objects go and puts them back when shown, keeping the models", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);

    set.stand(deliver(mockObjects(), [[0, mockLevelSpawnModel("crate")]]));
    set.show(new Set([ELevelSpawnCategory.ITEMS]));

    expect(sink.releaseObject.mock.calls).toEqual([[PROPS]]);
    expect(sink.releaseGeometry).not.toHaveBeenCalled();

    set.show(new Set([ELevelSpawnCategory.ITEMS, ELevelSpawnCategory.PROPS]));

    expect(listPut(sink)).toEqual([PROPS, ITEMS, PROPS]);
  });

  it("lets everything go for other objects, and for none", () => {
    const sink: TMockSink = mockSink();
    const set: LevelRenderSpawnSet = new LevelRenderSpawnSet(sink);
    const crate: ILevelSpawnModel = mockLevelSpawnModel("crate");

    set.stand(deliver(mockObjects(), [[0, crate]]));
    set.stand(deliver(mockObjects(), [[0, crate]]));

    expect(sink.releaseObject.mock.calls).toEqual([[PROPS], [ITEMS]]);
    expect(sink.releaseGeometry.mock.calls).toEqual([[LEVEL_RENDER_KEYS.spawnGeometry(0)]]);

    set.stand(null);

    expect(sink.releaseObject).toHaveBeenCalledTimes(4);
    expect(sink.releaseGeometry).toHaveBeenCalledTimes(2);
    expect(sink.releaseSurface).toHaveBeenCalledTimes(2);
  });
});
