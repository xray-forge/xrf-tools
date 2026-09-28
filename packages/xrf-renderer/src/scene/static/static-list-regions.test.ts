import { describe, expect, it, jest } from "@jest/globals";
import { storage } from "three/tsl";
import { BufferAttribute, BufferGeometry, StorageBufferAttribute } from "three/webgpu";

import { StaticArena } from "#/scene/static/static-arena";
import { StaticBatch } from "#/scene/static/static-batch";
import { StaticListRegions } from "#/scene/static/static-list-regions";
import { STATIC_SHADOW_VIEWS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticListSpace } from "#/uniforms/static-list-space";
import { EStaticPool } from "#/uniforms/static-pool";
import { EStaticView } from "#/uniforms/static-view";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createRegions(): {
  buffers: StaticDrawBuffers;
  regions: StaticListRegions;
  toBatch: (demand: number) => StaticBatch;
} {
  const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), {
    [EStaticPool.SHADOW_LIST]: 1,
    [EStaticPool.SURFACE_LIST]: 100,
  });
  const buffer: BufferGeometry = new BufferGeometry();
  let ids: number = 0;

  buffer.setAttribute("position", new BufferAttribute(new Float32Array(9), 3));

  const arena: StaticArena = new StaticArena(
    buffer,
    storage(new StorageBufferAttribute(new Uint32Array(2), 2), "uvec2", 1).toReadOnly(),
    storage(new StorageBufferAttribute(new Uint32Array(4), 4), "uvec4", 1).toReadOnly(),
    new StorageRetirement()
  );

  function toBatch(demand: number): StaticBatch {
    const batch: StaticBatch = new StaticBatch(arena, ids++, EStaticListSpace.SURFACES, [
      () => buffers.viewArgs[EStaticView.EARLY],
    ]);

    batch.put(0, demand);

    return batch;
  }

  return { buffers, regions: new StaticListRegions(buffers), toBatch };
}

function toRegion(buffers: StaticDrawBuffers, batch: StaticBatch): Array<number> {
  return Array.from((buffers.batchRegions.array as Uint32Array).subarray(batch.id * 4, batch.id * 4 + 3));
}

describe("StaticListRegions", () => {
  it("gives a batch a region of its demand with room to spare, kept while the demand fits it", () => {
    const { buffers, regions, toBatch } = createRegions();
    const batch: StaticBatch = toBatch(8);

    expect(regions.fit(batch)).toBe(true);
    expect(toRegion(buffers, batch)).toEqual([0, 10, EStaticListSpace.SURFACES]);

    const version: number = regions.version;

    batch.put(0, 10);

    expect(regions.fit(batch)).toBe(true);
    expect(regions.version).toBe(version);
    expect(regions.extent(EStaticListSpace.SURFACES)).toBe(10);
  });

  it("moves a region its batch outgrew, and frees the one it had", () => {
    const { buffers, regions, toBatch } = createRegions();
    const batch: StaticBatch = toBatch(8);

    regions.fit(batch);
    batch.put(0, 16);
    regions.fit(batch);

    expect(toRegion(buffers, batch)).toEqual([10, 20, EStaticListSpace.SURFACES]);
    expect(regions.use(EStaticListSpace.SURFACES)).toEqual({ capacity: 100, used: 20 });
  });

  it("keeps the region a batch had where the space cannot grow to a larger one", () => {
    const { buffers, regions, toBatch } = createRegions();
    const batch: StaticBatch = toBatch(8);

    buffers.storageLimit = 8 * (2 * 100 + STATIC_SHADOW_VIEWS);
    regions.fit(batch);
    batch.put(0, 200);

    expect(regions.fit(batch)).toBe(false);
    expect(toRegion(buffers, batch)).toEqual([0, 10, EStaticListSpace.SURFACES]);
    expect(buffers.capacity(EStaticPool.SURFACE_LIST)).toBe(100);
  });

  // Regions move whenever a batch outgrows one, so a fragmented space is the usual one.
  it("grows a fragmented space once, straight past its last region by the new one", () => {
    const { buffers, regions, toBatch } = createRegions();
    const batches: Array<StaticBatch> = Array.from({ length: 10 }, () => toBatch(8));
    const grow = jest.spyOn(buffers, "grow");

    batches.forEach((batch: StaticBatch) => regions.fit(batch));
    batches
      .filter((_: StaticBatch, index: number) => index % 2 === 0)
      .forEach((it: StaticBatch) => regions.release(it));

    const batch: StaticBatch = toBatch(16);

    expect(regions.fit(batch)).toBe(true);
    expect(grow.mock.calls).toEqual([[EStaticPool.SURFACE_LIST, 150]]);
    expect(toRegion(buffers, batch)).toEqual([100, 20, EStaticListSpace.SURFACES]);
  });

  it("leaves a released batch's region of nothing, drawing no vertex", () => {
    const { buffers, regions, toBatch } = createRegions();
    const batch: StaticBatch = toBatch(8);

    regions.fit(batch);
    regions.release(batch);
    regions.flush();

    expect(batch.region).toBeNull();
    expect(toRegion(buffers, batch)).toEqual([0, 0, EStaticListSpace.SURFACES]);
    expect(buffers.batchRegions.updateRanges).toEqual([{ count: 4, start: 0 }]);
  });
});
