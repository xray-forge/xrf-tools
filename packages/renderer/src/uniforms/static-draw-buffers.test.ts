import { describe, expect, it } from "@jest/globals";

import {
  EStaticPool,
  EStaticView,
  STATIC_BATCH_ARGUMENTS,
  STATIC_SHADOW_VIEWS,
  STATIC_SLOT_WORDS,
  STATIC_VIEWS,
  StaticDrawBuffers,
} from "#/uniforms/static-draw-buffers";

describe("StaticDrawBuffers", () => {
  it("grows the slots with every record written kept", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.SLOTS]: 2 });
    const { slots } = buffers;

    (buffers.slots.array as Uint32Array).set([4, 2, 9, 1, 3, 5, 0, 0], STATIC_SLOT_WORDS);
    buffers.grow(EStaticPool.SLOTS, 8);

    expect(buffers.slots.array).toHaveLength(8 * STATIC_SLOT_WORDS);
    expect(Array.from((buffers.slots.array as Uint32Array).subarray(8, 16))).toEqual([4, 2, 9, 1, 3, 5, 0, 0]);
    expect(buffers.takeRetired()).toEqual([slots]);
    expect(buffers.takeRetired()).toEqual([]);
  });

  it("grows the clusters with every range kept, and repoints the node every static shader reads them through", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.CLUSTERS]: 2 });

    (buffers.clusterRanges.array as Uint32Array).set([30, 128, 7, 1], 4);
    buffers.grow(EStaticPool.CLUSTERS, 16);

    expect(buffers.clusterSpheres.array).toHaveLength(64);
    expect(Array.from((buffers.clusterRanges.array as Uint32Array).subarray(4, 8))).toEqual([30, 128, 7, 1]);
    expect(buffers.clusterRangeWords.value).toBe(buffers.clusterRanges);
  });

  it("grows the batches with their regions kept and every view's arguments new", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.BATCHES]: 2 });
    const early = buffers.viewArgs[EStaticView.EARLY];

    (buffers.batchRegions.array as Uint32Array).set([100, 20, 0, 0], 4);
    buffers.grow(EStaticPool.BATCHES, 8);

    expect(Array.from((buffers.batchRegions.array as Uint32Array).subarray(4, 8))).toEqual([100, 20, 0, 0]);
    expect(buffers.viewArgs).toHaveLength(STATIC_VIEWS);
    expect(buffers.viewArgs[EStaticView.EARLY]).not.toBe(early);
    expect(buffers.viewArgs[EStaticView.EARLY].array).toHaveLength(8 * STATIC_BATCH_ARGUMENTS);
    expect(buffers.takeRetired()).toContain(early);
  });

  // The camera's two views take a surface list space each, then each shadow view a shadow one.
  it("lays every view's list out one after another, and moves the ones after a space that grew", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({
      [EStaticPool.SHADOW_LIST]: 10,
      [EStaticPool.SURFACE_LIST]: 100,
    });

    expect(buffers.toListBase(EStaticView.LATE)).toBe(100);
    expect(buffers.toListBase(EStaticView.SHADOW + 1)).toBe(210);
    expect(buffers.lists.array).toHaveLength((200 + 10 * STATIC_SHADOW_VIEWS) * 2);

    buffers.grow(EStaticPool.SURFACE_LIST, 300);

    expect(buffers.toListBase(EStaticView.SHADOW + 1)).toBe(610);
    expect(buffers.listEntries.value).toBe(buffers.lists);
    expect(buffers.candidates.array).toHaveLength(600);
  });

  it("bumps its layout with every growth, so shaders built over it are built again", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.PLACES]: 4, [EStaticPool.PYRAMID]: 4 });
    const layout: number = buffers.layout;

    buffers.grow(EStaticPool.PLACES, 8);
    buffers.grow(EStaticPool.PYRAMID, 8);

    expect(buffers.layout).toBe(layout + 2);
    expect(buffers.placeColumns.value).toBe(buffers.places);
  });

  it("limits each pool to what its widest buffer holds within a storage buffer", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();

    buffers.storageLimit = 1 << 20;

    expect(buffers.limit(EStaticPool.SLOTS)).toBe((1 << 20) / 32);
    expect(buffers.limit(EStaticPool.PLACES)).toBe(Math.floor((1 << 20) / 80));
    expect(buffers.limit(EStaticPool.SHADOW_LIST)).toBe(Math.floor((1 << 20) / (8 * STATIC_SHADOW_VIEWS)));
    expect(buffers.limit(EStaticPool.PYRAMID)).toBe((1 << 20) / 4);
  });

  it("hands on what else draws static draws gave up, with its own", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();
    const { rowSpheres } = buffers;

    buffers.retire([rowSpheres]);

    expect(buffers.takeRetired()).toEqual([rowSpheres]);
  });
});
