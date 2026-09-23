import { describe, expect, it } from "@jest/globals";
import { Matrix4 } from "three/webgpu";

import { StaticPlaces } from "#/scene/static/static-places";
import { STATIC_PLACE_COLUMNS, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";

/** Two places, one moved one metre along x, the other two, with hemisphere terms or without. */
function createInstances(hasHemi: boolean) {
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
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();
    const places: StaticPlaces = new StaticPlaces(buffers);
    const start: number = places.allocatePlaces(2) as number;
    const array = buffers.places.array as Float32Array;
    const at: number = (start + 1) * STATIC_PLACE_COLUMNS * 4;

    places.writePlaces(start, createInstances(true), new Matrix4().makeTranslation(0, 10, 0));

    expect([array[at + 12], array[at + 13]]).toEqual([2, 10]);
    expect([array[at + 16], array[at + 17]]).toEqual([0.75, 0.125]);
  });

  it("leaves the vertex hemi as it is for instances without terms of their own", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();
    const places: StaticPlaces = new StaticPlaces(buffers);

    places.writePlaces(places.allocatePlaces(2) as number, createInstances(false), new Matrix4());

    expect(Array.from((buffers.places.array as Float32Array).subarray(16, 18))).toEqual([1, 0]);
  });

  it("makes a row a place of one draw, listed from the rows' own start, and a freed row tests nothing", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();
    const places: StaticPlaces = new StaticPlaces(buffers);
    const start: number = places.allocateRows(2) as number;
    const version: number = places.version;

    places.writeRows(start, new Float32Array([0, 0, 0, 1, 5, 0, 0, 1]), 40, 7, 36);

    expect(Array.from((buffers.rowTargets.array as Uint32Array).subarray(4, 8))).toEqual([41, 7, start, 36]);
    expect(places.version).toBeGreaterThan(version);

    places.freeRows(start, 2);

    expect((buffers.rowSpheres.array as Float32Array)[7]).toBe(-1);
  });

  it("uploads what changed as one span a buffer", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers();
    const places: StaticPlaces = new StaticPlaces(buffers);

    places.writeRows(places.allocateRows(2) as number, new Float32Array(8), 0, 0, 3);
    places.flush();

    expect(buffers.rowTargets.updateRanges).toEqual([{ count: 8, start: 0 }]);
    expect(buffers.places.updateRanges).toEqual([]);
  });
});
