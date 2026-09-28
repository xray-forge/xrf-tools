import { describe, expect, it } from "@jest/globals";
import { Matrix4 } from "three/webgpu";

import { IRendererInstances } from "#/contract/scene/renderer-instances";
import { StaticPlaces } from "#/scene/static/static-places";
import { STATIC_PLACE_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

function createInstances(hasHemi: boolean): IRendererInstances {
  return {
    hemi: hasHemi ? new Float32Array([0.5, 0.25, 0.75, 0.125]) : undefined,
    transforms: new Float32Array([
      ...new Matrix4().makeTranslation(1, 0, 0).elements,
      ...new Matrix4().makeTranslation(2, 0, 0).elements,
    ]),
  };
}

describe("StaticPlaces", () => {
  it("stands each place where its object's matrix puts its instance, with its hemisphere terms", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const places: StaticPlaces = new StaticPlaces(buffers);
    const start: number = places.allocate(2) as number;
    const array: Float32Array = buffers.places.array as Float32Array;
    const at: number = (start + 1) * STATIC_PLACE_COLUMNS * 4;

    places.writePlaces(start, createInstances(true), new Matrix4().makeTranslation(0, 10, 0));

    expect([array[at + 12], array[at + 13]]).toEqual([2, 10]);
    expect([array[at + 16], array[at + 17]]).toEqual([0.75, 0.125]);
  });

  it("leaves the vertex hemi as it is for instances without terms of their own", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const places: StaticPlaces = new StaticPlaces(buffers);

    places.writePlaces(places.allocate(2) as number, createInstances(false), new Matrix4());

    expect(Array.from((buffers.places.array as Float32Array).subarray(16, 18))).toEqual([1, 0]);
  });

  it("grows with what is written and handed out kept, and bumps its version", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.PLACES]: 2 });
    const places: StaticPlaces = new StaticPlaces(buffers);
    const start: number = places.allocate(2) as number;

    places.writePlaces(start, createInstances(true), new Matrix4());

    expect(places.allocate(1)).toBeNull();

    const version: number = places.version;

    places.grow(8);

    expect(places.allocate(1)).toBe(2);
    expect([places.capacity, places.used]).toEqual([8, 3]);
    expect(places.version).toBeGreaterThan(version);
    expect((buffers.places.array as Float32Array)[STATIC_PLACE_COLUMNS * 4 + 12]).toBe(2);
  });

  // An instanced cluster's sphere is in its mesh's own space: the cull stands it in the place, scaled by the greatest.
  it("keeps each place's greatest scale, and gives a single draw a place of its own drawing its vertex hemi", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const places: StaticPlaces = new StaticPlaces(buffers);
    const start: number = places.allocate(3) as number;

    places.writePlaces(start, createInstances(true), new Matrix4().makeScale(1, 3, 2));
    places.writePlace(start + 2, new Matrix4().makeTranslation(7, 0, 0));

    const floats: Float32Array = buffers.places.array as Float32Array;

    expect(floats[STATIC_PLACE_COLUMNS * 4 * start + 19]).toBe(3);
    expect(
      Array.from(
        floats.subarray(STATIC_PLACE_COLUMNS * 4 * (start + 2) + 12, STATIC_PLACE_COLUMNS * 4 * (start + 2) + 13)
      )
    ).toEqual([7]);
    expect(
      Array.from(floats.subarray(STATIC_PLACE_COLUMNS * 4 * (start + 2) + 16, STATIC_PLACE_COLUMNS * 4 * (start + 3)))
    ).toEqual([1, 0, -1, 1]);
  });

  it("names each place's impostor from where its set starts, and none for a place without", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const places: StaticPlaces = new StaticPlaces(buffers);
    const start: number = places.allocate(2) as number;
    const floats: Float32Array = buffers.places.array as Float32Array;

    places.writePlaces(
      start,
      { ...createInstances(false), impostors: { indices: new Int32Array([3, -1]), key: "clump" } },
      new Matrix4(),
      40
    );

    expect(floats[STATIC_PLACE_COLUMNS * 4 * start + 18]).toBe(43);
    expect(floats[STATIC_PLACE_COLUMNS * 4 * (start + 1) + 18]).toBe(-1);
  });

  it("uploads what changed as one span", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const places: StaticPlaces = new StaticPlaces(buffers);

    places.writePlace(places.allocate(2) as number, new Matrix4());
    places.flush();

    expect(buffers.places.updateRanges).toEqual([{ count: STATIC_PLACE_COLUMNS * 4, start: 0 }]);
  });
});
