import { describe, expect, it, jest } from "@jest/globals";
import { BufferAttribute } from "three/webgpu";

import { IStorageDeviceFixture, mockStorageDevice } from "#/internals/device-fixtures";
import { createStorageBuffer } from "#/internals/storage-buffers";
import {
  STATIC_BATCH_ARGUMENTS,
  STATIC_SHADOW_VIEWS,
  STATIC_SLOT_WORDS,
  STATIC_VIEWS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { EStaticView } from "#/uniforms/static-view";
import { StorageRetirement } from "#/uniforms/storage-retirement";

/** Buffers of the first sizes given, and every buffer they hand to be freed. */
function createBuffers(initial: Partial<Record<EStaticPool, number>> = {}): {
  buffers: StaticDrawBuffers;
  retired: Array<BufferAttribute>;
} {
  const retirement: StorageRetirement = new StorageRetirement();
  const retired: Array<BufferAttribute> = [];

  jest
    .spyOn(retirement, "retire")
    .mockImplementation((attributes: Iterable<BufferAttribute>) => void retired.push(...attributes));

  return { buffers: new StaticDrawBuffers(retirement, initial), retired };
}

describe("StaticDrawBuffers", () => {
  it("grows the slots with every record written kept, and retires the buffer it replaced at once", () => {
    const { buffers, retired } = createBuffers({ [EStaticPool.SLOTS]: 2 });
    const { slots } = buffers;

    (buffers.slots.array as Uint32Array).set([4, 2, 9, 1, 3, 5, 0, 0], STATIC_SLOT_WORDS);
    buffers.grow(EStaticPool.SLOTS, 8);

    expect(buffers.slots.array).toHaveLength(8 * STATIC_SLOT_WORDS);
    expect(Array.from((buffers.slots.array as Uint32Array).subarray(8, 16))).toEqual([4, 2, 9, 1, 3, 5, 0, 0]);
    expect(retired).toEqual([slots]);
  });

  it("grows the clusters with every range kept, and repoints the node every static shader reads them through", () => {
    const { buffers } = createBuffers({ [EStaticPool.CLUSTERS]: 2 });

    (buffers.clusterRanges.array as Uint32Array).set([30, 128, 7, 1], 4);
    buffers.grow(EStaticPool.CLUSTERS, 16);

    expect(buffers.clusterSpheres.array).toHaveLength(64);
    expect(Array.from((buffers.clusterRanges.array as Uint32Array).subarray(4, 8))).toEqual([30, 128, 7, 1]);
    expect(buffers.clusterRangeWords.value).toBe(buffers.clusterRanges);
  });

  it("grows the batches with their regions kept and every view's arguments new", () => {
    const { buffers, retired } = createBuffers({ [EStaticPool.BATCHES]: 2 });
    const early: BufferAttribute = buffers.viewArgs[EStaticView.EARLY];

    (buffers.batchRegions.array as Uint32Array).set([100, 20, 0, 0], 4);
    buffers.grow(EStaticPool.BATCHES, 8);

    expect(Array.from((buffers.batchRegions.array as Uint32Array).subarray(4, 8))).toEqual([100, 20, 0, 0]);
    expect(buffers.viewArgs).toHaveLength(STATIC_VIEWS);
    expect(buffers.viewArgs[EStaticView.EARLY]).not.toBe(early);
    expect(buffers.viewArgs[EStaticView.EARLY].array).toHaveLength(8 * STATIC_BATCH_ARGUMENTS);
    expect(retired).toContain(early);
  });

  // The camera's two views take a surface list space each, then each shadow view a shadow one.
  it("lays every view's list out one after another, and moves the ones after a space that grew", () => {
    const { buffers, retired } = createBuffers({ [EStaticPool.SHADOW_LIST]: 10, [EStaticPool.SURFACE_LIST]: 100 });
    const { lists, candidates } = buffers;

    expect(buffers.toListBase(EStaticView.LATE)).toBe(100);
    expect(buffers.toListBase(EStaticView.SHADOW + 1)).toBe(210);
    expect(buffers.lists.array).toHaveLength((200 + 10 * STATIC_SHADOW_VIEWS) * 2);

    buffers.grow(EStaticPool.SURFACE_LIST, 300);

    expect(buffers.toListBase(EStaticView.SHADOW + 1)).toBe(610);
    expect(buffers.listEntries.value).toBe(buffers.lists);
    expect(buffers.candidates.array).toHaveLength(600);
    expect(retired).toEqual([lists, candidates]);
  });

  it("bumps its layout with every growth, so shaders built over it are built again", () => {
    const { buffers } = createBuffers({ [EStaticPool.PLACES]: 4, [EStaticPool.PYRAMID]: 4 });
    const layout: number = buffers.layout;

    buffers.grow(EStaticPool.PLACES, 8);
    buffers.grow(EStaticPool.PYRAMID, 8);

    expect(buffers.layout).toBe(layout + 2);
    expect(buffers.placeColumns.value).toBe(buffers.places);
  });

  it("limits each pool to what its widest buffer holds within a storage buffer", () => {
    const { buffers } = createBuffers({ [EStaticPool.SHADOW_LIST]: 0, [EStaticPool.SURFACE_LIST]: 0 });

    buffers.storageLimit = 1 << 20;

    expect(buffers.limit(EStaticPool.SLOTS)).toBe((1 << 20) / 32);
    expect(buffers.limit(EStaticPool.PLACES)).toBe(Math.floor((1 << 20) / 80));
    expect(buffers.limit(EStaticPool.SHADOW_LIST)).toBe(Math.floor((1 << 20) / (8 * STATIC_SHADOW_VIEWS)));
    expect(buffers.limit(EStaticPool.PYRAMID)).toBe((1 << 20) / 4);
  });

  // Every view's list is one buffer, so each space may take only what the other leaves of it.
  it("limits each list space to what the other space leaves of the buffer they share", () => {
    const { buffers } = createBuffers({ [EStaticPool.SHADOW_LIST]: 1000, [EStaticPool.SURFACE_LIST]: 5000 });
    const entries: number = (1 << 20) / 8;

    buffers.storageLimit = 1 << 20;

    const surfaces: number = buffers.limit(EStaticPool.SURFACE_LIST);
    const shadows: number = buffers.limit(EStaticPool.SHADOW_LIST);

    expect(surfaces).toBe(Math.floor((entries - 1000 * STATIC_SHADOW_VIEWS) / 2));
    expect(shadows).toBe(Math.floor((entries - 2 * 5000) / STATIC_SHADOW_VIEWS));
    // Either space grown to its limit leaves the whole list within the storage limit.
    expect((2 * surfaces + STATIC_SHADOW_VIEWS * 1000) * 8).toBeLessThanOrEqual(1 << 20);
    expect((2 * 5000 + STATIC_SHADOW_VIEWS * shadows) * 8).toBeLessThanOrEqual(1 << 20);
  });

  it("limits a list space to none where the other already takes the whole buffer", () => {
    // A kilobyte is 128 entries: the camera's two views of 64 each.
    const { buffers } = createBuffers({ [EStaticPool.SHADOW_LIST]: 1, [EStaticPool.SURFACE_LIST]: 64 });

    buffers.storageLimit = 1 << 10;

    expect(buffers.limit(EStaticPool.SHADOW_LIST)).toBe(0);
  });

  // The lists alone are tens of megabytes of zeroes the CPU never reads, on a level the size of Pripyat.
  it("lets the arrays of what the GPU alone writes go once up, grown ones too, and keeps what the pools write", () => {
    const device: IStorageDeviceFixture = mockStorageDevice();
    const retirement: StorageRetirement = new StorageRetirement();
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(retirement, { [EStaticPool.PYRAMID]: 16 });

    // Made by three as something first binds them.
    [buffers.lists, buffers.pyramid, buffers.slots, buffers.viewArgs[EStaticView.EARLY]].forEach(
      (attribute: BufferAttribute) => createStorageBuffer(device.renderer, attribute, attribute.array.byteLength)
    );
    retirement.free(device.renderer);

    expect(buffers.lists.array).toHaveLength(2);
    expect(buffers.pyramid.array).toHaveLength(1);
    expect(buffers.viewArgs[EStaticView.EARLY].array).toHaveLength(STATIC_BATCH_ARGUMENTS);
    expect(buffers.slots.array.length).toBeGreaterThan(4);

    buffers.grow(EStaticPool.PYRAMID, 64);
    createStorageBuffer(device.renderer, buffers.pyramid, 256);
    retirement.free(device.renderer);

    expect(buffers.pyramid.array).toHaveLength(1);
    expect(buffers.pyramid.count).toBe(64);
  });
});
