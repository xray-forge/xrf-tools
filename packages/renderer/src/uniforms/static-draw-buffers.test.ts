import { describe, expect, it } from "@jest/globals";

import { EStaticPool, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

describe("StaticDrawBuffers", () => {
  it("grows the slots with everything written kept, new spheres drawing nothing, and the matrices' node repointed", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.SLOTS]: 2 });
    const { args, models } = buffers;

    (buffers.args.array as Uint32Array).set([12, 1, 30, 7, 1], 5);
    (buffers.models.array as Float32Array)[16] = 3;
    buffers.grow(EStaticPool.SLOTS, 8);

    expect(buffers.capacity(EStaticPool.SLOTS)).toBe(8);
    expect(buffers.args.array).toHaveLength(40);
    expect(Array.from((buffers.args.array as Uint32Array).subarray(5, 10))).toEqual([12, 1, 30, 7, 1]);
    expect((buffers.models.array as Float32Array)[16]).toBe(3);
    expect((buffers.spheres.array as Float32Array)[7 * 4 + 3]).toBe(-1);
    expect(buffers.modelColumns.value).toBe(buffers.models);
    expect(buffers.takeRetired()).toEqual(expect.arrayContaining([args, models]));
    expect(buffers.takeRetired()).toEqual([]);
  });

  it("grows the rows with both halves of the list, and repoints the list's node", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers({ [EStaticPool.ROWS]: 4 });

    buffers.grow(EStaticPool.ROWS, 16);

    expect(buffers.visible.array).toHaveLength(32);
    expect(buffers.rowSpheres.array).toHaveLength(64);
    expect(buffers.visiblePlaces.value).toBe(buffers.visible);
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

    expect(buffers.limit(EStaticPool.SLOTS)).toBe((1 << 20) / 64);
    expect(buffers.limit(EStaticPool.PLACES)).toBe(Math.floor((1 << 20) / 80));
    expect(buffers.limit(EStaticPool.PYRAMID)).toBe((1 << 20) / 4);
  });
});
