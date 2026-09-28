import { describe, expect, it } from "@jest/globals";

import { StaticRows } from "#/scene/static/static-rows";
import { STATIC_NO_BAND, STATIC_NO_LOD, StaticDrawBuffers } from "#/uniforms/static-draw-buffers";
import { EStaticPool } from "#/uniforms/static-pool";
import { StorageRetirement } from "#/uniforms/storage-retirement";

describe("StaticRows", () => {
  it("makes a row a place of one draw, which stands the draw's clusters there, and a freed row tests nothing", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const rows: StaticRows = new StaticRows(buffers);
    const start: number = rows.allocate(2) as number;
    const version: number = rows.version;

    rows.write(start, new Float32Array([0, 0, 0, 1, 5, 0, 0, 1]), 40, 7);

    expect(Array.from((buffers.rowTargets.array as Uint32Array).subarray(4, 8))).toEqual([41, 7, 0, 0]);
    expect(Array.from((buffers.rowLods.array as Uint32Array).subarray(0, 2))).toEqual([STATIC_NO_LOD, STATIC_NO_BAND]);
    expect(rows.version).toBeGreaterThan(version);

    rows.free(start, 2);

    expect((buffers.rowSpheres.array as Float32Array)[7]).toBe(-1);
    expect(rows.used).toBe(0);
  });

  it("grows with what is written and handed out kept, and bumps its version", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement(), { [EStaticPool.ROWS]: 2 });
    const rows: StaticRows = new StaticRows(buffers);

    rows.write(rows.allocate(2) as number, new Float32Array([0, 0, 0, 1, 5, 0, 0, 1]), 0, 3);

    const version: number = rows.version;

    rows.grow(8);

    expect(rows.allocate(6)).toBe(2);
    expect([rows.capacity, rows.used, rows.extent]).toEqual([8, 8, 8]);
    expect(rows.version).toBeGreaterThan(version);
    expect(Array.from((buffers.rowTargets.array as Uint32Array).subarray(4, 8))).toEqual([1, 3, 0, 0]);
    // A row added by the growth tests nothing until it is written.
    expect((buffers.rowSpheres.array as Float32Array)[7 * 4 + 3]).toBe(-1);
  });

  it("uploads what changed as one span a buffer", () => {
    const buffers: StaticDrawBuffers = new StaticDrawBuffers(new StorageRetirement());
    const rows: StaticRows = new StaticRows(buffers);

    rows.write(rows.allocate(2) as number, new Float32Array(8), 0, 0);
    rows.flush();

    expect(buffers.rowTargets.updateRanges).toEqual([{ count: 8, start: 0 }]);
    expect(buffers.rowLods.updateRanges).toEqual([{ count: 4, start: 0 }]);
  });
});
